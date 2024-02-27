import { useSelector } from "react-redux";
import { toast } from "react-toastify";
import axios from "axios";
import { coreEndpoint, validateTransaction } from "../../utils/helper";
import {
  selectUserAddress,
  selectOwshen,
  selectNetwork,
  selectIsTest,
  setReceivedCoins,
} from "../../store/containerSlice";
import { ethers, toBigInt } from "ethers";
import { utils } from "web3";
import { useApprove } from "../../hooks/useApprove";

export const useTransactionModalApi = (tokenContract) => {
  const address = useSelector(selectUserAddress);
  const OwshenWallet = useSelector(selectOwshen);
  const network = useSelector(selectNetwork);
  const isTest = useSelector(selectIsTest);
  const { approve, allowance } = useApprove(
    tokenContract,
    address,
    OwshenWallet.contract_address,
    OwshenWallet.dive_abi
  );

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
      address,
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
          if (Number(allowance) < Number(tokenAmount))
            await approve(to_wei_token_amount);
          const tx = await contract.deposit(
            pubKey,
            ephemeral,
            tokenContract,
            utils.toBigInt(to_wei_token_amount),
          );
          await tx.wait();
          axios.get(`${coreEndpoint}/coins`).then((result) => {
            setReceivedCoins(result.data.coins);
            setIsOpen(false);
          });
        } catch (error) {
          toast.error("Error while transferring tokens!");

          console.log(error, "Error while transferring tokens!");
        }
      })
      .catch((error) => {
        return toast.error(`Internal server error: ${error}`);
      });
  };

  const send = async (
    destOwshenWallet,
    tokenContract,
    tokenAmount,
    chainId,
    findMatchingCoin
  ) => {
    const errorMessage = validateTransaction(
      address,
      destOwshenWallet,
      tokenContract,
      tokenAmount,
      network,
      chainId
    );
    if (errorMessage) {
      return toast.error(errorMessage);
    }

    const selectedCoint = findMatchingCoin();

    const options = {
      gasLimit: 5000000,
    };
    const to_wei_token_amount = utils.toWei(Number(tokenAmount), "ether");
    axios

      .get(`${coreEndpoint}/send`, {
        params: {
          index: selectedCoint.index,
          address: OwshenWallet.wallet,
          new_amount: to_wei_token_amount,
          receiver_address: destOwshenWallet,
        },
      })
      .then(async (result) => {
        let abi = OwshenWallet.contract_abi;
        let reciver_commitment = result.data.receiver_commitment;
        let sender_commitment = result.data.sender_commitment;
        let provider = new ethers.BrowserProvider(window.ethereum);

        let contract = new ethers.Contract(
          OwshenWallet.contract_address,
          abi,
          provider
        );
        let signer = await provider.getSigner();
        contract = contract.connect(signer);

        const proof = [
          result.data.proof.a,
          result.data.proof.b,
          result.data.proof.c,
        ];

        const rax = utils.toBigInt(result.data.receiver_ephemeral.x);
        const ray = utils.toBigInt(result.data.receiver_ephemeral.y);

        const receiver_ephemeral = [rax, ray];

        const sax = utils.toBigInt(result.data.sender_ephemeral.x);
        const say = utils.toBigInt(result.data.sender_ephemeral.y);

        const sender_ephemeral = [sax, say];

        try {
          const txResponse = await contract.send(
            result.data.root,
            proof,
            receiver_ephemeral,
            sender_ephemeral,
            [result.data.nullifier, utils.toBigInt(0)],
            [sender_commitment, reciver_commitment],
            result.data.obfuscated_receiver_token_address,
            result.data.obfuscated_sender_token_address,
            result.data.obfuscated_receiver_amount,
            result.data.obfuscated_sender_amount,
            true,
            options
          );
          console.log("Transaction response", txResponse);
          const txReceipt = await txResponse.wait();
          console.log("Transaction receipt", txReceipt);
        } catch (error) {
          toast.error("Error while transferring tokens!");
          console.log(error, "Error while transferring tokens!");
        }
      })
      .catch((error) => {
        return toast.error(`Internal server error: ${error}`);
      });
  };

  const withdrawal = async (index, owshen, address, setIsOpen, tokenAmount) => {
    if (!address) return toast.error("Connect your wallet first!");
    // if (tokenAmount > trueAmount(MaxBalanceOfWithdraw)) {
    //   return toast.error(
    //     "your entered amount for withdraw is grater than max value of the selected token"
    //   );
    // }
    //Todo: its should be with dynamic decimal
    const desireAmount = utils.toWei(Number(tokenAmount), "ether");
    const options = {
      gasLimit: 5000000,
    };
    axios
      .get(`${coreEndpoint}/withdraw`, {
        params: {
          index: index,
          address: address, // TODO: change it to user modal eth address
          desire_amount: desireAmount,
        },
      })
      .then(async (result) => {
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

        const proof = [
          result.data.proof.a,
          result.data.proof.b,
          result.data.proof.c,
        ];

        const ax = utils.toBigInt(result.data.ephemeral.x);
        const ay = utils.toBigInt(result.data.ephemeral.y);

        const ephemeral = [ax, ay];
        try {
          const txResponse = await contract.withdraw(
            result.data.root,
            proof,
            ephemeral,
            [result.data.nullifier, utils.toBigInt(0)],
            result.data.token,
            toBigInt(desireAmount),
            result.data.obfuscated_remaining_amount,
            address,
            commitment,
            options
          );
          console.log("Transaction response", txResponse);
          const txReceipt = await txResponse.wait();
          console.log("Transaction receipt", txReceipt);
          setIsOpen(false);
        } catch (error) {
          toast.error("Error while transferring tokens!");
          setIsOpen(false);
        }
      })
      .catch((error) => {
        return toast.error(`Internal server error: ${error}`);
      });
  };
  return { newGetStealth, send, withdrawal };
};
