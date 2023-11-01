import { ethers, formatUnits } from "ethers";

export const openSend = () => {
  let elem = document.getElementById("send-modal");
  elem.style.display = "block";
};

export const closeSend = () => {
  let elem = document.getElementById("send-modal");
  elem.style.display = "none";
};

export const getERC20Balance = async (tokenAddress, contractAddress, ABI) => {
  const provider = new ethers.BrowserProvider(window.ethereum);
  const contract = new ethers.Contract(tokenAddress, ABI, provider);
  const balance = await contract.balanceOf(contractAddress);
  return formatUnits(balance, 18); // Assumes the token has 18 decimal places
};
