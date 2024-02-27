import React, { useState, useEffect } from "react";
import Web3 from "web3";
import Web3Modal from "web3modal";
import WalletIcon from "../../pics/icons/account_balance_wallet.png";
import SelectNetwork from "../SelectNetwork";
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
    "border lg:w-52 w-full rounded-xl px-3 py-3   ease-in-out duration-300 flex items-center justify-around";

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

  return (
    <>
      {account ? (
        <>
          <SelectNetwork />
          <button
            className={`${buttonClass} bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] dark:bg-indigo-950  lg:ml-3 mb-3 lg:mb-0`}
            onClick={disconnectWallet}
          >
            Disconnect Wallet
          </button>
        </>
      ) : (
        <button
          className={`${buttonClass} bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] dark:bg-indigo-950  `}
          onClick={connectWallet}
        >
          <img src={WalletIcon} width="20px" alt="WalletIcon" />
          <p>Connect Wallet</p>
        </button>
      )}

      {/* <Dropdown label={shortenAddress(account)} />
      <div className="border rounded-2xl px-3 py-2 mt-3  bg-[#BBDCFBCC] ease-in-out duration-300  flex">
        {shortenAddress(account)}
        <img className="ml-2" src={CopyIcon} />{" "}
      </div>{" "} */}
    </>
  );
};
export default Web3ModalComponent;
