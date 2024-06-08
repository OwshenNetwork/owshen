import React, { useEffect } from "react";
import {
  setIsWalletConnected,
  // selectIsWalletConnected,
} from "../../store/containerSlice";
import { useDispatch } from "react-redux";
// import { toast } from "react-toastify";

const WalletConnectionChecker = () => {
  const wallet = window.ethereum;
  const dispatch = useDispatch();
  // const isConnected = useSelector(selectIsWalletConnected);
  useEffect(() => {
    const checkWalletConnection = async () => {
      if (window.ethereum) {
        try {
          // Request account access
          await window.ethereum.request({ method: "eth_requestAccounts" });
          dispatch(setIsWalletConnected(true));
        } catch (error) {
          console.error("User denied account access", error);
          dispatch(setIsWalletConnected(false));
        }
      } else {
        console.log(
          "Non-Ethereum browser detected. You should consider trying MetaMask!"
        );
        dispatch(setIsWalletConnected(false));
      }
    };

    checkWalletConnection();
  }, [dispatch, wallet]);

  return (
    <div>
      {
        //!isConnected && toast.warning(" No wallet detected. Please connect your wallet to proceed."
        // <p className="mx-auto text-center py-5 bg-yellow-500 text-2xl ">
        //   No wallet detected. Please connect your wallet to proceed.
        // </p>
        // )
      }
    </div>
  );
};

export default WalletConnectionChecker;
