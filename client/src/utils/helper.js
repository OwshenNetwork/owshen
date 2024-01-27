import { ethers, formatUnits, toBigInt } from "ethers";
import { toast } from "react-toastify";
import { getDecimalByContractAddress } from "./Currencies";

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
};
