import { ethers } from "ethers";
import { useState, useEffect } from "react";

export const useApprove = (tokenAddress, ownerAddress, spenderAddress, ABI) => {
  const [allowance, setAllowance] = useState(0);
  const [loading, setLoading] = useState(true);
  const [contract, setContract] = useState(null);

  useEffect(() => {
    const initializeContract = async () => {
      try {
        const provider = new ethers.BrowserProvider(window.ethereum);
        const signer = await provider.getSigner();
        if (tokenAddress && ABI) {
          const contract = new ethers.Contract(tokenAddress, ABI, signer);
          setContract(contract);
        }
      } catch (error) {
        console.error("Error initializing contract:", error);
        // Handle the error appropriately, e.g., show a message to the user
      }
    };
    initializeContract();
  }, [tokenAddress, ABI]);

  useEffect(() => {
    if (!contract) {
      return;
    }
    const fetchAllowance = async () => {
      if (ownerAddress) {
        const allowance = await contract.allowance(
          ownerAddress,
          spenderAddress
        );
        setAllowance(allowance);
        setLoading(false);
      }
    };
    fetchAllowance();
  }, [contract, ownerAddress, spenderAddress]);

  const approve = async (amount) => {
    if (!contract) {
      throw new Error("Contract is not initialized");
    }
    const tx = await contract.approve(spenderAddress, amount);
    await tx.wait();
  };

  return { allowance, loading, approve };
};
