import { ethers, formatUnits, toBigInt } from "ethers";

export const openSend = () => {
  let elem = document.getElementById("send-modal");
  elem.style.display = "block";
};

export const closeSend = () => {
  let elem = document.getElementById("send-modal");
  elem.style.display = "none";
};

export const getERC20Balance = async (tokenAddress, userAddress, ABI) => {
  const provider = new ethers.BrowserProvider(window.ethereum);
  const contract = new ethers.Contract(tokenAddress, ABI, provider);
  const balance = await contract.balanceOf(userAddress);
  return formatUnits(balance, 18); // Assumes the token has 18 decimal places
};

export const trueAmount = (val) => {
  return Number(toBigInt(val).toString()) / Math.pow(10, 18);
};
