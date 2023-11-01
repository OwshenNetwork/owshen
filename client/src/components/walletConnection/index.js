import React, { useState, useEffect } from "react";
import Web3 from "web3";
import Web3Modal from "web3modal";

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

  if (!account) {
    return (
      <button className="connect-btn" onClick={connectWallet}>
        Connect Wallet
      </button>
    );
  }

  return (
    <div className="disconnect-container">
      <p>Account: {account}</p>
      <button onClick={disconnectWallet}>Disconnect Wallet</button>
    </div>
  );
};

export default Web3ModalComponent;
