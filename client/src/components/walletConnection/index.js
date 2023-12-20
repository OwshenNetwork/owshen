import React, { useState, useEffect } from "react";
import Web3 from "web3";
import Web3Modal from "web3modal";
import WalletIcon from "../../pics/icons/account_balance_wallet.png";
import CopyIcon from "../../pics/icons/copy.png";
import Dropdown from "../dropDown";

import "./style.css";
import { setUserDetails } from "../../store/containerSlice";
import { useDispatch, useSelector, UseSelector } from "react-redux";
import { selectIsTest } from "../../store/containerSlice";
import InProgress from "../Modal/InProgress";
const Web3ModalComponent = () => {
  const [provider, setProvider] = useState(null);
  const [account, setAccount] = useState(null);
  const [isInprogress, setIsInprogress] = useState(false);
  const [selectNetwork, setSelectNetWork] = useState("");

  const netWorkOptions = [
    {
      title: "network1",
      value: "network1",
    },
    { title: "network2", value: "network2" },
  ];
  const isTest = useSelector(selectIsTest);

  const dispatch = useDispatch();

  useEffect(() => {
    if (window.ethereum) {
      window.ethereum.on("accountsChanged", (accounts) => {
        setAccount(accounts[0]);
      });
    }
  }, []);

  const connectWallet = async () => {
    // if (isTest) {
    //   return setIsInprogress(true);
    // }
    const web3Modal = new Web3Modal();
    const _provider = await web3Modal.connect();
    const web3 = new Web3(_provider);

    const accounts = await web3.eth.getAccounts();
    setProvider(_provider);
    setAccount(accounts[0]);
    dispatch(setUserDetails({ address: accounts[0] }));
  };
  const buttonClass =
    "border w-52 rounded-2xl px-3 py-2   ease-in-out duration-300 flex items-center justify-around";

  const disconnectWallet = async () => {
    if (provider.close) {
      await provider.close();

      // If the cached provider is not cleared, WalletConnect will automatically
      // connect to the previously connected wallet when we try to call 'connect' again!
      await window.localStorage.removeItem("walletconnect");
    }

    setProvider(null);
    setAccount(null);
  };

  function shortenAddress(address) {
    const firstPart = address.substring(0, 6);
    const lastPart = address.substring(address.length - 4);
    return `${firstPart}...${lastPart}`;
  }

  return (
    <>
      <InProgress isOpen={isInprogress} setIsOpen={setIsInprogress} />
      {account ? (
        <>
          <button
            className={`${buttonClass} bg-[#EBEDEF]  hover:bg-[#BBDCFBCC]  mr-3`}
            onClick={disconnectWallet}
          >
            Disconnect Wallet
          </button>

          {!isTest && (
            <Dropdown
              label="network1"
              options={netWorkOptions}
              select={setSelectNetWork}
              style="!bg-gray-200 !text-white !py-3 !rounded-xl border-0 "
            />
          )}
        </>
      ) : (
        <button
          className={`${buttonClass} bg-[#EBEDEF]  hover:bg-[#BBDCFBCC]  `}
          onClick={connectWallet}
        >
          <img src={WalletIcon} width="20px" />
          <p>Connect Wallet</p>
        </button>
      )}
    </>
  );
};
export default Web3ModalComponent;
