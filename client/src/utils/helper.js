import { ethers, formatUnits, toBigInt } from "ethers";
import { toast } from "react-toastify";

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

export const trueAmount = (val) => {
  return Number(toBigInt(val).toString()) / Math.pow(10, 18);
};

export const getRound = (num) => {
  return Number.parseFloat(num.toFixed(3));
};
