import { useEffect, useState } from "react";
import Modal from ".";
import Dropdown from "../DropDown";
import {
  selectUserAddress,
  selectOwshen,
  selectReceivedCoins,
  selectIsTest,
} from "../../store/containerSlice";
import { getERC20Balance, chainIdOfWallet } from "../../utils/helper";
import { useSelector } from "react-redux";
import Logo from "../../pics/icons/logo.png";
import MetaMaskLogo from "../../pics/icons/metaMask.png";
import { toast } from "react-toastify";
import { currencies } from "../../utils/Currencies";
import { trueAmount } from "../../utils/helper";
import {
  getNetworkNameByChainId,
  networkDetails,
} from "../../utils/networkDetails";
import { useTransactionModalApi } from "../../api/hooks/useTransactionModalApi";
import ReactLoading from "react-loading";
const TransactionModal = ({
  setTokenContract,
  tokenContract,
  transactionType,
  isOpen,
  setIsOpen,
  selectedCoin,
  isDataSet,
}) => {
  const address = useSelector(selectUserAddress);
  const OwshenWallet = useSelector(selectOwshen);
  const receivedCoins = useSelector(selectReceivedCoins);
  const [destOwshenWallet, setDstOwshenWallet] = useState("");
  const [tokenAmount, setTokenAmount] = useState(0);
  const [, setWalletName] = useState("");
  const [tokenOptions, setTokenOptions] = useState([]);
  const [MaxBalanceOfWithdraw, setMaxBalanceOfWithdraw] = useState("");
  const [selectedContract, setSelectedContract] = useState("");
  const [chainId, setChainId] = useState(null);
  const [selectTokenLabel, SetSelectTokenLabel] = useState("Choose your token");
  const [selectWalletLabel, SetSelectWalletLabel] = useState("Source wallet");
  const [isLoading, setIsLoading] = useState(false);
  const isTest = useSelector(selectIsTest);
  const { send, withdrawal, newGetStealth } =
    useTransactionModalApi(tokenContract);

  const walletOptions = [
    { title: "Your Owshen Account", value: "Your Owshen Account", img: Logo },
  ];
  useEffect(() => {
    if (!isTest) {
      walletOptions.push({
        title: "Your Ethereum Account",
        value: "Your Ethereum Account",
        img: MetaMaskLogo,
      });
    }
  });

  const getChainId = async () => {
    const ChainId = await chainIdOfWallet();
    setChainId(ChainId);
  };
  useEffect(() => {
    getChainId();
  }, []);
  useEffect(() => {
    if (OwshenWallet && chainId) {
      setContract(chainId);
      if (selectedContract) {
        const newTokenOptions = OwshenWallet.token_contracts?.networks?.[
          selectedContract
        ].map(({ symbol, token_address }, id) => {
          const img = currencies[symbol].img;

          return { title: symbol, value: token_address, img: img };
        });
        setTokenOptions(newTokenOptions);
      }
    }
  }, [OwshenWallet, chainId, selectedContract]);
  useEffect(() => {
    setMaxBalanceOfWithdraw(selectedCoin?.amount);
  }, [selectedCoin]);
  const setContract = (chainId) => {
    if (chainId) {
      const networkName = getNetworkNameByChainId(chainId);
      const networkContract = networkDetails[networkName]?.contractName;
      return setSelectedContract(networkContract);
    }
    setSelectedContract("Sepolia");
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
    for (let coin of receivedCoins) {
      if (
        trueAmount(coin.amount, coin.uint_token) > Number(tokenAmount) &&
        String(coin.uint_token) === String(tokenContract)
      ) {
        return coin;
      }
    }

    return toast.error("No matching coin is found");
  };

  const handleSend = async () => {
    setIsLoading(true); // Set isLoading to true at the beginning of the method

    if (destOwshenWallet.length !== 69) {
      setIsLoading(false); // Set isLoading to false if there's an error
      return toast.error(
        "Please make sure your destination wallet address is correct"
      );
    }
    const firstPartOfAddress = destOwshenWallet.slice(0, 4);
    if (firstPartOfAddress !== "OoOo") {
      return toast.error(
        "your destination wallet address must start with 'OoOo'"
      );
      setIsLoading(false)
    }
    try {
      if (OwshenWallet.wallet === destOwshenWallet) {
        await newGetStealth(
          destOwshenWallet,
          tokenContract,
          tokenAmount,
          chainId,
          setIsOpen
        );
      } else {
        await send(
          destOwshenWallet,
          tokenContract,
          tokenAmount,
          chainId,
          findMatchingCoin
        );
      }
    } catch (error) {
      console.error("Error during transaction:", error);
      toast.error("An error occurred during the transaction.");
    } finally {
      setIsLoading(false); // Set isLoading to false after the transaction is complete
    }
  };
  const tokenAmountHandler = (e) => {
    const newVal = e.target.value;
    // Regular expression to match only numbers or an empty string
    const regex = /^(\d+)?$/;
    // Test the input value against the regular expression
    if (regex.test(newVal)) {
      setTokenAmount(newVal);
    }
  };
  useEffect(() => {
    if (!isOpen) {
      setDstOwshenWallet("");
      setTokenAmount(0);
      SetSelectTokenLabel("Choose your token");
      SetSelectWalletLabel("Source wallet");
    }
  }, [isOpen]);

  const loading = (
    <ReactLoading type="bars" color="#2481D7" height={30} width={30} />
  );
  return (
    <Modal title={transactionType} isOpen={isOpen} setIsOpen={setIsOpen}>
      <p className="mt-5">
        {transactionType === "Withdraw"
          ? "Privately withdraw ERC-20 tokens from your Owshen address!"
          : "Privately send ERC-20 tokens to other Owshen users!"}
      </p>
      {transactionType !== "Withdraw" && (
        <>
          <div className="px-3 flex justify-between items-center relative">
            <p className="absolute top-5 text-blue-500 left-5 text-xl">
              Destination
            </p>
            <input
              className="rounded py-7 px-2 bg-white dark:bg-indigo-950 my-4 border w-full border-blue-500 focus:border-blue-500 active:border-blue-500 "
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
              label={selectWalletLabel}
              options={walletOptions}
              select={setWalletName}
              style={`py-5 !w-60`}
              setLabel={SetSelectWalletLabel}
            />
          </div>
        </>
      )}
      <div className="px-3 flex justify-between items-center mt-3">
        <label>
          <b>Token: </b>
        </label>
        <Dropdown
          label={isDataSet ? "DIVE" : selectTokenLabel}
          options={tokenOptions}
          select={setTokenContract}
          style={`py-5 ${isDataSet ? "pointer-events-none" : ""}!w-60`}
          setLabel={SetSelectTokenLabel}
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
            className="rounded py-5 px-2 bg-white dark:bg-indigo-950 my-4 border w-60 text-center"
            placeholder="Enter amount"
            onChange={tokenAmountHandler}
            type="number"
            value={tokenAmount}
          />
        </>
      </div>
      <button
        disabled={isLoading}
        onClick={() =>
          transactionType === "Withdraw"
            ? withdrawal(
                selectedCoin?.index,
                OwshenWallet,
                address,
                setIsOpen,
                tokenAmount
              )
            : handleSend()
        }
        className="border border-blue-400 bg-blue-200 text-blue-600 rounded-lg px-6 mt-3 font-bold py-1"
      >
        {isLoading
          ? loading
          : transactionType === "Withdraw"
          ? transactionType
          : "Send"}
      </button>
    </Modal>
  );
};

export default TransactionModal;
