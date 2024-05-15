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
  const rounded = Number.parseFloat(num.toFixed(4));
  return rounded % 1 === 0 ? rounded + ".0" : rounded;
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

export const coreEndpoint =
  process.env.REACT_APP_OWSHEN_ENDPOINT || "http://localhost:9000";

export const chainIdOfWallet = async () => {
  try {
    const web3Modal = new Web3Modal();
    const provider = await web3Modal.connect();
    const web3 = new Web3(provider);
    const chainId = await web3.eth.getChainId(); // Get the chainId
    return Number(chainId);
  } catch (error) {
    console.error("Error getting chain ID:", error);
  }
};

export const validateTransaction = (
  destOwshenWallet,
  tokenContract,
  tokenAmount,
  network,
  chainId
) => {
  if (!destOwshenWallet)
    return "Destination wallet address is missing. Please enter the recipient's wallet address.";
  if (!tokenContract)
    return "Token selection is required. Please choose the token you wish to send.";
  if (!tokenAmount || tokenAmount === 0)
    return "Token amount is not specified. Please enter the amount of tokens you want to send.";
  if (network.chainId !== chainId) {
    SwitchNetwork(network.name);
    return `Your wallet is currently on a different network. Please switch to the ${network.name} network to continue.`;
  }
  return null; // No error
};

export const copyWalletWords = (CopyWords) => {
  navigator.clipboard.writeText(CopyWords);
  toast.success(
    "Your wallet phrases has been copied to your clipboard successfully!"
  );
};
