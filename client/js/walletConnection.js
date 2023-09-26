import Web3Modal from "web3-modal";
import WalletConnectProvider from "@walletconnect/web3-provider";
import { ethers } from "ethers";

const providerOptions = {
  walletconnect: {
    package: WalletConnectProvider,
    options: {
      infuraId: "44a735d4387b48a2b5024f16b3159611",
    },
  },
};

const web3Modal = new Web3Modal({
  network: "goerli", // optional
  cacheProvider: true, // optional
  providerOptions, // required
});

const connectWallet = async () => {
  const provider = await web3Modal.connect();
  return new ethers.providers.Web3Provider(provider);
};

document.getElementById("btn-connect").addEventListener("click", connectWallet);
