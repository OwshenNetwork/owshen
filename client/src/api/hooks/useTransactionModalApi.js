import { useSelector } from "react-redux";
import { toast } from "react-toastify";
import axios from "axios";
import {
  coreEndpoint,
  trueAmount,
  validateTransaction,
} from "../../utils/helper";
import {
  selectUserAddress,
  selectOwshen,
  selectNetwork,
  selectIsTest,
  setReceivedCoins,
  selectReceivedCoins,
} from "../../store/containerSlice";
import { ethers, toBigInt } from "ethers";
import { utils } from "web3";
import { useApprove } from "../../hooks/useApprove";
import { generate_witness } from "../../utils/proof";
import { groth16 } from "snarkjs";

export const useTransactionModalApi = (tokenContract) => {
  const address = useSelector(selectUserAddress);
  const receivedCoins = useSelector(selectReceivedCoins);
  const OwshenWallet = useSelector(selectOwshen);
  const network = useSelector(selectNetwork);
  const isTest = useSelector(selectIsTest);

  const witnessCalculator = `${coreEndpoint}/witness/witness_calculator.js`;
  const zk_file = `${coreEndpoint}/zk/coin_withdraw_0001.zkey`;
  const wasm = `${coreEndpoint}/witness/coin_withdraw.wasm`;

  const { approve, allowance } = useApprove(
    tokenContract,
    address,
    OwshenWallet.contract_address,
    OwshenWallet.dive_abi
  );

  const options = {
    gasLimit: 5000000,
  };

  const newGetStealth = async (
    destOwshenWallet,
    tokenContract,
    tokenAmount,
    chainId,
    setIsOpen
  ) => {
    if (isTest) {
      return toast.error("You can't send to yourself!");
    }
    const errorMessage = validateTransaction(
      destOwshenWallet,
      tokenContract,
      tokenAmount,
      network,
      chainId
    );
    if (errorMessage) {
      return toast.error(errorMessage);
    }
    await axios
      .get(`${coreEndpoint}/stealth`, {
        params: { address: destOwshenWallet },
      })
      .then(async (result) => {
        let abi = OwshenWallet.contract_abi;
        let provider = new ethers.BrowserProvider(window.ethereum);
        let contract = new ethers.Contract(
          OwshenWallet.contract_address,
          abi,
          provider
        );

        let signer = await provider.getSigner();
        contract = contract.connect(signer);

        const ax = utils.toBigInt(result.data.address.x);
        const ay = utils.toBigInt(result.data.address.y);
        const ex = utils.toBigInt(result.data.ephemeral.x);
        const ey = utils.toBigInt(result.data.ephemeral.y);

        const pubKey = [ax, ay];
        const ephemeral = [ex, ey];
        const to_wei_token_amount = utils.toWei(Number(tokenAmount), "ether");
        try {
          if (Number(allowance) < to_wei_token_amount)
            await approve(to_wei_token_amount);

          const tx = await contract.deposit(
            pubKey,
            ephemeral,
            tokenContract,
            utils.toBigInt(to_wei_token_amount),
            OwshenWallet.wallet,
            options
          );
          await tx.wait();
          axios.get(`${coreEndpoint}/coins`).then((result) => {
            setReceivedCoins(result.data.coins);
          });
          setIsOpen(false);
        } catch (error) {
          toast.error("Error while transferring tokens!");

          console.log(error, "Error while transferring tokens!");
        }
      })
      .catch((error) => {
        return toast.error(`Internal server error: ${error}`);
      });
  };

  const formatProof = (proof) => {
    return [
      [proof.pi_a[0], proof.pi_a[1]],
      [
        [proof.pi_b[0][1], proof.pi_b[0][0]],
        [proof.pi_b[1][1], proof.pi_b[1][0]],
      ],
      [proof.pi_c[0], proof.pi_c[1]],
    ];
  };

  const send = async (
    destOwshenWallet,
    tokenContract,
    tokenAmount,
    chainId,
    setIsOpen
  ) => {
    const errorMessage = validateTransaction(
      destOwshenWallet,
      tokenContract,
      tokenAmount,
      network,
      chainId
    );
    if (errorMessage) {
      return toast.error(errorMessage);
    }

    let selectedCoin;
    for (let coin of receivedCoins) {
      if (
        trueAmount(coin.amount, coin.uint_token) > Number(tokenAmount) &&
        String(coin.uint_token) === String(tokenContract)
      ) {
        selectedCoin = coin;
        break;
      }
    }

    if (!selectedCoin) {
      return toast.error("No matching coin is found!");
    }

    const options = {
      gasLimit: 5000000,
    };
    const to_wei_token_amount = utils.toWei(Number(tokenAmount), "ether");
    axios

      .get(`${coreEndpoint}/send`, {
        params: {
          index: selectedCoin.index,
          address: OwshenWallet.wallet,
          new_amount: to_wei_token_amount,
          receiver_address: destOwshenWallet,
          memo: OwshenWallet.wallet,
        },
      })
      .then(async (result) => {
        let abi = OwshenWallet.contract_abi;
        let receiver_commitment = result.data.receiver_commitment;
        let sender_commitment = result.data.sender_commitment;

        let proof =
          OwshenWallet.mode === "Windows"
            ? formatProof(
                (
                  await groth16.prove(
                    new Uint8Array(await (await fetch(zk_file)).arrayBuffer()),
                    await generate_witness(
                      wasm,
                      witnessCalculator,
                      result.data.proof?.JsonInput,
                      wasm
                    )
                  )
                ).proof
              )
            : [
                result.data.proof.Proof.a,
                result.data.proof.Proof.b,
                result.data.proof.Proof.c,
              ];

        let provider = new ethers.BrowserProvider(window.ethereum);

        let contract = new ethers.Contract(
          OwshenWallet.contract_address,
          abi,
          provider
        );
        let signer = await provider.getSigner();
        contract = contract.connect(signer);

        const rax = utils.toBigInt(result.data.receiver_ephemeral.x);
        const ray = utils.toBigInt(result.data.receiver_ephemeral.y);

        const receiver_ephemeral = [rax, ray];

        const sax = utils.toBigInt(result.data.sender_ephemeral.x);
        const say = utils.toBigInt(result.data.sender_ephemeral.y);

        const sender_ephemeral = [sax, say];

        try {
          const txResponse = await contract.send(
            result.data.checkpoint_head,
            result.data.latest_values_commitment_head,
            proof,
            receiver_ephemeral,
            sender_ephemeral,
            [result.data.nullifier, utils.toBigInt(0)],
            [sender_commitment, receiver_commitment],
            result.data.obfuscated_receiver_token_address,
            result.data.obfuscated_sender_token_address,
            result.data.obfuscated_receiver_amount,
            result.data.obfuscated_sender_amount,
            true,
            result.data.memo,
            options
          );
          console.log("Transaction response", txResponse);
          const txReceipt = await txResponse.wait();
          console.log("Transaction receipt", txReceipt);
          setIsOpen(false);
        } catch (error) {
          setIsOpen(false);
          toast.error("Error while transferring tokens!");
          console.log(error, "Error while transferring tokens!");
        }
      })
      .catch((error) => {
        console.log("error ", error);
        setIsOpen(false);
        return toast.error(`Internal server error: ${error}`);
      });
  };

  const withdrawal = async (
    index,
    owshen,
    address,
    setIsOpen,
    tokenAmount,
    setIsLoading
  ) => {
    setIsLoading(true);
    if (!address) {
      setIsLoading(false); // Stop loading if address is not provided
      return toast.error(
        "Your wallet is not connected. Please connect your wallet to proceed."
      );
    }
    if (!tokenAmount || tokenAmount === 0) {
      setIsLoading(false);
      return toast.error(
        "Token amount is not specified. Please enter the amount of tokens you want to withdraw."
      );
    }

    const desireAmount = utils.toWei(Number(tokenAmount), "ether");
    axios
      .get(`${coreEndpoint}/withdraw`, {
        params: {
          index,
          address,
          owshen_address: owshen.wallet, // TODO: change it to user modal eth address
          desire_amount: desireAmount,
        },
      })
      .then(async (result) => {
        let proof =
          OwshenWallet.mode === "Windows"
            ? formatProof(
                (
                  await groth16.prove(
                    new Uint8Array(await (await fetch(zk_file)).arrayBuffer()),
                    await generate_witness(
                      wasm,
                      witnessCalculator,
                      result.data.proof?.JsonInput,
                      wasm
                    )
                  )
                ).proof
              )
            : [
                result.data.proof.Proof.a,
                result.data.proof.Proof.b,
                result.data.proof.Proof.c,
              ];

        let abi = owshen.contract_abi;
        let commitment = result.data.commitment;
        let provider = new ethers.BrowserProvider(window.ethereum);

        let contract = new ethers.Contract(
          owshen.contract_address,
          abi,
          provider
        );

        let signer = await provider.getSigner();
        contract = contract.connect(signer);

        const ax = utils.toBigInt(result.data.ephemeral.x);
        const ay = utils.toBigInt(result.data.ephemeral.y);

        const ephemeral = [ax, ay];
        try {
          const txResponse = await contract.withdraw(
            result.data.checkpoint_head,
            result.data.latest_values_commitment_head,
            proof,
            ephemeral,
            [result.data.nullifier, 0],
            result.data.token,
            toBigInt(desireAmount),
            result.data.obfuscated_remaining_amount,
            address,
            commitment,
            OwshenWallet.wallet
          );
          console.log("Transaction response", txResponse);
          const txReceipt = await txResponse.wait();
          console.log("Transaction receipt", txReceipt);
          setIsOpen(false);
          setIsLoading(false); // Stop loading if address is not provided
        } catch (error) {
          setIsLoading(false); // Stop loading if address is not provided
          toast.error("Error while transferring tokens!");
          setIsOpen(false);
          console.error("Error in ethers.js operations:", error);
        }
      })
      .catch((error) => {
        setIsLoading(false); // Stop loading if address is not provided
        setIsOpen(false);
        return toast.error(`Internal server error: ${error}`);
      })
      .finally(() => {
        setIsLoading(false); // Stop loading if address is not provided
        setIsOpen(false);
      });
  };
  return { newGetStealth, send, withdrawal };
};
