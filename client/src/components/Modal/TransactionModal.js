import { useEffect, useState, useCallback } from "react";
import Modal from ".";
import Dropdown from "../DropDown";
import {
  selectUserAddress,
  selectOwshen,
  selectIsTest,
} from "../../store/containerSlice";
import { getERC20Balance, chainIdOfWallet } from "../../utils/helper";
import { useSelector } from "react-redux";
import Logo from "../../pics/logo.png";
import MetaMaskLogo from "../../pics/icons/metaMask.png";
import { toast } from "react-toastify";
import { currencies, getNameByContractAddress } from "../../utils/Currencies";
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
  const [destOwshenWallet, setDstOwshenWallet] = useState("");
  const [tokenAmount, setTokenAmount] = useState(null);
  const [tokenOptions, setTokenOptions] = useState([]);
  const [MaxBalanceOfWithdraw, setMaxBalanceOfWithdraw] = useState("");
  const [selectedContract, setSelectedContract] = useState("");
  const [chainId, setChainId] = useState(null);
  const [selectTokenLabel, SetSelectTokenLabel] = useState("Choose your token");
  const [loadingText, SetLoadingText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [defaultVal, setDefaultVal] = useState(true);

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

  useEffect(() => {
    if (isOpen) {
      setIsLoading(false);
    }
  }, [isOpen]);

  useEffect(() => {
    const getChainId = async () => {
      const ChainId = await chainIdOfWallet();
      setChainId(ChainId);
    };
    getChainId();
  });
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
  const shuffleText = useCallback(() => {
    const randomLoadingTexts = [
      "Processing your request...",
      "This may take some time...",
      "Please hold on! Proof generation takes time...",
      "We're almost done...",
    ];
    const index = Math.floor(Math.random() * randomLoadingTexts.length);
    SetLoadingText(randomLoadingTexts[index]);
  }, []);
  const handleSend = async () => {
    setInterval(shuffleText, 8000);
    setIsLoading(true); // Set isLoading to true at the beginning of the method

    if (destOwshenWallet.length !== 69) {
      setIsLoading(false); // Set isLoading to false if there's an error
      return toast.error(
        "Please make sure the destination wallet address is valid."
      );
    }
    const firstPartOfAddress = destOwshenWallet.slice(0, 4);
    if (firstPartOfAddress !== "OoOo") {
      toast.error("The destination wallet address must start with 'OoOo'");
      return setIsLoading(false);
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
        setInterval(shuffleText, 8000);
        await send(
          destOwshenWallet,
          tokenContract,
          tokenAmount,
          chainId,
          setIsOpen
        );
      }
    } catch (error) {
      console.error("Error while processing the transaction:", error);
      toast.error("An error occurred while processing the transaction.");
    } finally {
      setIsLoading(false); // Set isLoading to false after the transaction is complete
    }
  };
  const handleWithdraw = async () => {
    setInterval(shuffleText, 8000);
    try {
      await withdrawal(
        selectedCoin?.index,
        OwshenWallet,
        address,
        setIsOpen,
        tokenAmount,
        setIsLoading
      );
    } catch (error) {
      console.error("Error during withdrawal:", error);
      // You can add more error handling here if needed
    }
  };
  const tokenAmountHandler = (e) => {
    const newVal = e.target.value;
    // Regular expression to match only numbers or an empty string
    const regex = /^[0-9]*\.?[0-9]*$/; // Test the input value against the regular expression
    if (regex.test(newVal)) {
      setTokenAmount(newVal);
    }
  };

  useEffect(() => {
    if (!isOpen) {
      setDstOwshenWallet("");
      setTokenAmount(null);
      SetSelectTokenLabel("DIVE");
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
            {/* <p className="absolute top-5 text-blue-500 left-5 text-xl">
              Destination
            </p> */}
            <input
              className="rounded-lg text-center   py-3 px-2 bg-white dark:bg-indigo-950 my-4 border w-full border-blue-500 focus:border-blue-500 active:border-blue-500 "
              onChange={(e) => setDstOwshenWallet(e.target.value)}
              type="text"
              placeholder="Destination"
              value={destOwshenWallet}
            />
          </div>
        </>
      )}
      <div className="flex justify-center items-center w-full px-3">
        <div className="flex justify-between items-center relative w-4/6 lg:w-5/6">
          {/* <button
          onClick={() => {
            transactionType === "Withdraw"
              ? MaxBalanceOfWithdrawHandler(MaxBalanceOfWithdraw)
              : setMaxBalance();
          }}
          className="border rounded-3xl px-3 absolute -bottom-2 left-4 border-blue-500 text-blue-600"
        >
          <small>Max</small>
        </button> */}
          <>
            <input
              className="rounded-lg rounded-r-none py-3 px-2 bg-white dark:bg-indigo-950 my-4 border w-full text-center border-gray-400"
              placeholder="Amount"
              onChange={tokenAmountHandler}
              type="string"
              value={tokenAmount}
            />
          </>
        </div>
        <div className="flex justify-between items-center w-2/6 lg:w-1/6">
          {transactionType === "Withdraw" ? (
            <span className="py-3 !w-full rounded-lg  border rounded-l-none border-gray-400 flex">
              <img className="w-6 mr-2 ml-5" src={Logo} alt="logo" />{" "}
              {getNameByContractAddress(selectedCoin?.uint_token)}
            </span>
          ) : (
            <Dropdown
              label={isDataSet ? "DIVE" : selectTokenLabel}
              options={tokenOptions}
              select={setTokenContract}
              setDefaultVal={setDefaultVal}
              defaultVal={defaultVal}
              style={`py-3 !w-full   rounded-l-none ${
                isDataSet ? "pointer-events-none" : ""
              }`}
              setLabel={SetSelectTokenLabel}
            />
          )}
        </div>
      </div>

      <button
        disabled={isLoading}
        onClick={() =>
          transactionType === "Withdraw" ? handleWithdraw() : handleSend()
        }
        className="border border-blue-400 bg-blue-200 text-blue-600 rounded-lg px-6 mt-3 font-bold py-1 "
      >
        {isLoading
          ? loading
          : transactionType === "Withdraw"
          ? transactionType
          : "Send"}
      </button>
      <div className="my-2">{isLoading && loadingText}</div>
    </Modal>
  );
};

export default TransactionModal;
