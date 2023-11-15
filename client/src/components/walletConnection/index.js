import React, { useState, useEffect } from "react";
import Web3 from "web3";
import Web3Modal from "web3modal";
import WalletIcon from "../../pics/icons/account_balance_wallet.svg";
import CopyIcon from "../../pics/icons/copy.svg";

import "./style.css";
import { setUserDetails } from "../../store/containerSlice";
import { useDispatch } from "react-redux";

const Web3ModalComponent = () => {
  const [provider, setProvider] = useState(null);
  const [account, setAccount] = useState(null);

  const dispatch = useDispatch();

  useEffect(() => {
    if (window.ethereum) {
      window.ethereum.on("accountsChanged", (accounts) => {
        setAccount(accounts[0]);
      });
    }
  }, []);

  const connectWallet = async () => {
    const web3Modal = new Web3Modal();
    const _provider = await web3Modal.connect();
    const web3 = new Web3(_provider);

    const accounts = await web3.eth.getAccounts();
    setProvider(_provider);
    setAccount(accounts[0]);
    dispatch(setUserDetails({ address: accounts[0] }));
  };
  const buttonClass =
    "border w-52 rounded-2xl px-3 py-2 mt-3  ease-in-out duration-300 justify-center flex";

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

  if (!account) {
    return (
      <button
        className={`${buttonClass} bg-[#EBEDEF]  hover:bg-[#BBDCFBCC]  flex items-center justify-around`}
        onClick={connectWallet}
      >
        <img  src={WalletIcon} width="20px" />
        <p>Connect Wallet</p>
      </button>
    );
  }

  return (
    <div>
      {/* <div className="border rounded-2xl px-3 py-2 mt-3  bg-[#BBDCFBCC] ease-in-out duration-300  flex">
        {shortenAddress(account)}
        <img className="ml-2" src={CopyIcon} /> </div> */}
      <button
        className={`${buttonClass} bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] `}
        onClick={disconnectWallet}
      >
        Disconnect Wallet
      </button>
    </div>
  );
};

export default Web3ModalComponent;
