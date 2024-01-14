import { useEffect, useState } from "react";
import Modal from "./Modal";
import axios from "axios";
import Dropdown from "../DropDown";
import { ethers, toBigInt } from "ethers";
import { utils } from "web3";
import {
  selectUserAddress,
  selectOwshen,
  setReceivedCoins,
  selectReceivedCoins,
  selectNetwork,
} from "../../store/containerSlice";
import { useAccount } from "wagmi";
import { getERC20Balance } from "../../utils/helper";
import { useApprove } from "../../hooks/useApprove";
import { useSelector } from "react-redux";
import Logo from "../../pics/icons/logo.png";
import MetaMaskLogo from "../../pics/icons/metaMask.png";
import { toast } from "react-toastify";
import { currencies } from "../../utils/Currencies";
import { trueAmount } from "../../utils/helper";

const TransactionModal = ({
  setTokenContract,
  tokenContract,
  transactionType,
  isOpen,
  setIsOpen,
  selectedCoin,
  isDataSet,
}) => {
  const coreEndpoint =
    process.env.REACT_APP_OWSHEN_ENDPOINT || "http://127.0.0.1:9000";
  const address = useSelector(selectUserAddress);
  const OwshenWallet = useSelector(selectOwshen);
  const receivedcoins = useSelector(selectReceivedCoins);
  const network = useSelector(selectNetwork);
  const { chainId } = useAccount();
  const [destOwshenWallet, setDstOwshenWallet] = useState("");
  const [tokenAmount, setTokenAmount] = useState(0);
  const [walletName, setWalletName] = useState("");
  const [tokenOptions, setTokenOptions] = useState([]);
  const [MaxBalanceOfWithdraw, setMaxBalanceOfWithdraw] = useState("");
  const [selectedContract, setSelectedContract] = useState("");

  const walletOptions = [
    {
      title: "Your Ethereum Account",
      value: "Your Ethereum Account",
      img: MetaMaskLogo,
    },
    { title: "Your Owshen Account", value: "Your Owshen Account", img: Logo },
  ];
  useEffect(() => {
    if (OwshenWallet && chainId) {
      setContract(chainId);
      const newTokenOptions = OwshenWallet.token_contracts?.networks?.[
        selectedContract
      ].map(({ symbol, token_address }, id) => {
        const img = currencies[symbol].img;

        return { title: symbol, value: token_address, img: img };
      });
      setTokenOptions(newTokenOptions);
    }
  }, [OwshenWallet, chainId]);
  useEffect(() => {
    setMaxBalanceOfWithdraw(selectedCoin?.amount);
  }, [selectedCoin]);
  const { approve, allowance } = useApprove(
    tokenContract,
    address,
    OwshenWallet.contract_address,
    OwshenWallet.dive_abi
  );
  const setContract = (chainId) => {
    switch (chainId) {
      case 1337:
        setSelectedContract("localhost");
        break;
      case 11155111:
        setSelectedContract("Sepolia");
        break;
      case 5:
        setSelectedContract("ethereum_goerli");
        break;
      default:
        setSelectedContract("Sepolia");
    }
  };
  const setMaxBalance = async () => {
    if (tokenContract && address && OwshenWallet.dive_abi) {
      const value = await getERC20Balance(
        tokenContract,
        address,
        OwshenWallet.dive_abi
      );
      return setTokenAmount(value);
    }
  };
  const MaxBalanceOfWithdrawHandler = async (maxValue) => {
    const val = trueAmount(maxValue);
    return setTokenAmount(val);
  };

  const findMatchingCoin = () => {
    for (let coin of receivedcoins) {
      if (
        trueAmount(coin.amount) > Number(tokenAmount) &&
        String(coin.uint_token) === String(tokenContract)
      ) {
        return coin;
      }
    }

    return toast.error("No matching coin is found");
  };
  const getStealth = async () => {
    if (!address) return toast.error("Connect your wallet first");
    if (!destOwshenWallet) return toast.error("Enter your Destination");
    if (!tokenContract) return toast.error("Select your token");
    if (!tokenAmount) return toast.error("enter amount of token");
    if (network.chainId !== chainId)
      return toast.error(
        `please change your wallet network to ${network.name}`
      );
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
            address,
            OwshenWallet.contract_address
          );
          await tx.wait();
          axios.get(`${coreEndpoint}/coins`).then((result) => {
            setReceivedCoins(result.data.coins);
            setIsOpen(false);
          });
        } catch (error) {
          toast.error("Error while getting approve or deposit flow");

          console.log(error, "Error while getting approve or deposit flow");
        }
      })
      .catch((error) => {
        return toast.error(`internal server error: ${error}`);
      });
  };
  const send = async () => {
    if (!address) return toast.error("Connect your wallet first");
    if (!destOwshenWallet) return toast.error("Enter your Destination");
    if (!tokenContract) return toast.error("Select your token");
    if (!tokenAmount) return toast.error("enter amount of token");
    if (network.chainId !== chainId)
      return toast.error(
        `please change your wallet network to ${network.name}`
      );

    const selectedCoint = findMatchingCoin();

    const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;
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
        let commitment1 = result.data.sender_commitment;
        let commitment2 = result.data.receiver_commitment;
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
            result.data.nullifier,
            proof,
            receiver_ephemeral,
            sender_ephemeral,
            commitment1,
            commitment2,
            result.data.token,
            result.data.obfuscated_receiver_amount,
            result.data.obfuscated_sender_amount,
            true,
            options
          );
          console.log("Transaction response", txResponse);
          const txReceipt = await txResponse.wait();
          console.log("Transaction receipt", txReceipt);
        } catch (error) {
          toast.error("Error while getting withdraw flow");
          console.log(error, "Error while getting withdraw flow");
        }
      })
      .catch((error) => {
        return toast.error(`internal server error: ${error}`);
      });
  };

  const callSend = async () => {
    OwshenWallet.wallet === destOwshenWallet
      ? await getStealth()
      : await send();
  };

  const withdrawal = async (index, owshen, address) => {
    if (!address) return toast.error("Connect your wallet first");
    // if (tokenAmount > trueAmount(MaxBalanceOfWithdraw)) {
    //   return toast.error(
    //     "your entered amount for withdraw is grater than max value of the selected token"
    //   );
    // }
    //Todo: its should be with dynamic decimal
    const desireAmount = utils.toWei(Number(tokenAmount), "ether");
    const coreEndpoint =
      process.env.REACT_APP_OWSHEN_ENDPOINT || "http://127.0.0.1:9000";
    const options = {
      gasLimit: 5000000,
    };
    axios
      .get(`${coreEndpoint}/withdraw`, {
        params: {
          index: index,
          address: owshen.wallet,
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
            result.data.nullifier,
            ephemeral,
            proof,
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
          toast.error("Error while getting withdraw flow");
          setIsOpen(false);
        }
      })
      .catch((error) => {
        return toast.error(`internal server error: ${error}`);
      });
  };
  return (
    <Modal title={transactionType} isOpen={isOpen} setIsOpen={setIsOpen}>
      <p className="mt-5">
        {transactionType === "Withdraw"
          ? "Privately Withdraw tokens to Owshen users!"
          : "Privately send ERC-20 tokens to Owshen users!"}
      </p>
      {transactionType !== "Withdraw" && (
        <>
          <div className="px-3 flex justify-between items-center relative">
            <p className="absolute top-5 text-blue-500 left-5 text-xl">
              Destination
            </p>
            <input
              className="rounded py-7 px-2 bg-white my-4 border w-full border-blue-500 focus:border-blue-500 active:border-blue-500 "
              onChange={(e) => setDstOwshenWallet(e.target.value)}
              type="text"
              value={destOwshenWallet}
            />
          </div>
          <div className="px-3 flex justify-between items-center">
            <label>
              <b>From: </b>
            </label>
            <Dropdown
              label="choose your wallet"
              options={walletOptions}
              select={setWalletName}
              style="py-5"
            />
          </div>
        </>
      )}
      <div className="px-3 flex justify-between items-center mt-3">
        <label>
          <b>token: </b>
        </label>
        <Dropdown
          label={isDataSet ? "DIVE" : "choose your token"}
          options={tokenOptions}
          select={setTokenContract}
          style={`py-5 ${isDataSet ? "pointer-events-none" : ""}`}
        />
      </div>
      <div className="px-3 flex justify-between items-center relative">
        <button
          onClick={() => {
            transactionType === "Withdraw"
              ? MaxBalanceOfWithdrawHandler(MaxBalanceOfWithdraw)
              : setMaxBalance();
          }}
          className="border rounded-3xl px-3 absolute -bottom-2 left-4 border-blue-500 text-blue-600"
        >
          <small> max</small>
        </button>
        <>
          <label>
            <b>Amount:</b>
          </label>

          <input
            className="rounded py-5 px-2 bg-white my-4 border w-60 text-center"
            placeholder="Enter amount"
            onChange={(e) => setTokenAmount(e.target.value)}
            type="number"
            value={tokenAmount}
          />
        </>
      </div>
      <button
        onClick={() =>
          transactionType === "Withdraw"
            ? withdrawal(selectedCoin?.index, OwshenWallet, address)
            : callSend()
        }
        className="border border-blue-400 bg-blue-200 text-blue-600 rounded-lg px-6 mt-3 font-bold py-1"
      >
        {transactionType === "Withdraw" ? transactionType : "Send"}
      </button>
    </Modal>
  );
};

export default TransactionModal;
