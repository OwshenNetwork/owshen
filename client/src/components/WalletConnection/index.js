import React, { useEffect } from "react";
import Web3 from "web3";
import Web3Modal from "web3modal";
import { toast } from "react-toastify";
import WalletIcon from "../../pics/icons/account_balance_wallet.png";
import SelectNetwork from "../SelectNetwork";
import { setUserDetails, selectUserAddress } from "../../store/containerSlice";
import { useDispatch, useSelector } from "react-redux";
const Web3ModalComponent = () => {
  const dispatch = useDispatch();
  const account = useSelector(selectUserAddress);

  useEffect(() => {
    if (window.ethereum) {
      window.ethereum.on("accountsChanged", (accounts) => {
        dispatch(setUserDetails({ address: accounts[0] }));
      });
    }
  }, [dispatch]);

  const connectWallet = async () => {
    try {
      const web3Modal = new Web3Modal();
      const _provider = await web3Modal.connect();
      const web3 = new Web3(_provider);

      const accounts = await web3.eth.getAccounts();
      dispatch(setUserDetails({ address: accounts[0] }));
    } catch (error) {
      console.error("Error connecting wallet:", error);
      // Handle the error appropriately, e.g., show a message to the user
      return toast.error(
        "No wallet detected. Please connect your wallet to proceed."
      );
    }
  };
  const buttonClass =
    "border lg:w-52 w-full rounded-xl px-3 py-3   ease-in-out duration-300 flex items-center justify-around";

  const disconnectWallet = async () => {
    dispatch(setUserDetails({ address: null }));
  };

  return (
    <>
      {account ? (
        <>
          <SelectNetwork />
          <button
            className={`${buttonClass} bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] dark:bg-indigo-950  lg:ml-3 mt-3 lg:mt-0`}
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
