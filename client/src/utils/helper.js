import { ethers, formatUnits, toBigInt } from "ethers";
import { toast } from "react-toastify";
import { getDecimalByContractAddress } from "./Currencies";
import Web3 from "web3";
import Web3Modal from "web3modal";

export const getERC20Balance = async (tokenAddress, userAddress, ABI) => {
  const provider = new ethers.BrowserProvider(window.ethereum);
  const contract = new ethers.Contract(tokenAddress, ABI, provider);
  const balance = await contract.balanceOf(userAddress);
  return formatUnits(balance, 18); // Assumes the token has 18 decimal places
};

export const shortenAddress = (address) => {
  const firstPart = address.substring(0, 6);
  const lastPart = address.substring(address?.length - 4);
  return `${firstPart}...${lastPart}`;
};

export const copyWalletAddress = (owshenWalletWallet) => {
  navigator.clipboard.writeText(owshenWalletWallet);
  toast.success("You wallet address has been copied!");
};

export const trueAmount = (val, uintToken) => {
  if (uintToken) {
    return (
      Number(toBigInt(val).toString()) /
      Math.pow(10, getDecimalByContractAddress(uintToken))
    );
  }
  return Number(toBigInt(val).toString()) / Math.pow(10, 18);
};

export const getRound = (num) => {
  return Number.parseFloat(num.toFixed(3));
};

export const SwitchNetwork = async (networkName) => {
  const metamaskChainIds = {
    Sepolia: "0xaa36a7",
    Localhost: "0x539",
    Goerli: "0x5",
  };
  const testId = metamaskChainIds[networkName];

  // Get the current chain ID
  const web3 = new Web3(window.ethereum);
  const currentChainId = await web3.eth.getChainId();

  // Only proceed if the requested network is different from the current one
  if (testId !== currentChainId) {
    try {
      await window.ethereum.request({
        method: "wallet_switchEthereumChain",
        params: [
          {
            chainId: testId,
          },
        ],
      });
    } catch (error) {
      console.error("Failed to switch Ethereum chain:", error);
    }
  }
};

export const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;

export const chainIdOfWallet = async () => {
  try {
    const web3Modal = new Web3Modal();
    const provider = await web3Modal.connect();
    const web3 = new Web3(provider);
    const chainId = await web3.eth.getChainId(); // Get the chainId
    return Number(chainId);
  } catch (error) {
    console.error('Error getting chain ID:', error);
    throw error; // Re-throw the error so it can be handled by the calling code
  }
};

export const validateTransaction = (address, destOwshenWallet, tokenContract, tokenAmount, network, chainId) => {
  if (!address) return "Connect your wallet first!";
  if (!destOwshenWallet) return "Enter your destination!";
  if (!tokenContract) return "Select your token!";
  if (!tokenAmount) return "Enter amount of token!";
  if (network.chainId !== chainId) {
    SwitchNetwork(network.name);
    return `Please change your wallet network to ${network.name}`;
  }
  return null; // No error
};