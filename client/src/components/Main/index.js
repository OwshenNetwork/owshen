import React, { useEffect, useState } from "react";
import axios from "axios";
import {
  selectUserAddress,
  setOwshen,
  setReceivedCoins,
  setReceivedCoinsLoading,
  setIsTest,
  selectOwshen,
  selectIsTest,
} from "../../store/containerSlice";
import { useDispatch, useSelector } from "react-redux";
import { useAccount } from "wagmi";
import { toast } from "react-toastify";
import { Tooltip } from "react-tooltip";
import InProgress from "../Modal/InProgress";
import TransactionModal from "../Modal/TransactionModal";
import { shortenAddress, copyWalletAddress } from "../../utils/helper";

import "../../styles/main.css";
import CopyIcon from "../../pics/icons/copy.png";
import SendIcon from "../../pics/icons/send.png";
import SwapIcon from "../../pics/icons/swap.png";

const Main = ({ children }) => {
  const coreEndpoint =
    process.env.REACT_APP_OWSHEN_ENDPOINT || "http://127.0.0.1:9000";
  const address = useSelector(selectUserAddress);
  const accountData = useAccount();
  const chainId = accountData ? accountData.chainId : undefined;
  const OwshenWallet = useSelector(selectOwshen);
  const isTest = useSelector(selectIsTest);
  const dispatch = useDispatch();

  const [tokenContract, setTokenContract] = useState("");
  const [isOpen, setIsOpen] = useState(false);
  const [isInprogress, setIsInprogress] = useState(false);
  const [isOpenWithdraw, setIsOpenWithdraw] = useState(false);

  useEffect(() => {
    // This code will run whenever `tokenContract` changes
    if (tokenContract) {
      dispatch(
        setOwshen({ type: "SET_SELECT_TOKEN_CONTRACT", payload: tokenContract })
      );
    }
  }, [tokenContract]); // Add `tokenContract` as a dependency

  useEffect(() => {
    //TODO: proper flow for set network
    setChainId();
  }, [
    chainId,
    OwshenWallet.wallet,
    OwshenWallet.contract_address,
    coreEndpoint,
  ]);
  const setChainId = async () => {
    if (!chainId) {
      return toast.error("please connect your wallet");
    }
    let chain_id = chainId;
    if (chainId === 5) {
      chain_id = "0x5";
    }
    await axios
      .post(`${coreEndpoint}/set-network`, null, {
        params: { chain_id },
      })
      .then((response) => {
        console.log("Response:", response.data);
        GetCoins();
        GetInfo();
      })
      .catch((error) => {
        console.error("Error:", error);
      });
  };
  const GetCoins = () => {
    dispatch(setReceivedCoinsLoading(true));
    const coinsIntervalId = setInterval(() => {
      axios.get(`${coreEndpoint}/coins`).then((result) => {
        dispatch(
          setReceivedCoins({
            type: "SET_RECEIVED_COINS",
            payload: result.data.coins,
          })
        );
        dispatch(setReceivedCoinsLoading(result.data.syncing));
      });
    }, 5000);
    return () => clearInterval(coinsIntervalId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  };
  const GetInfo = () => {
    axios.get(`${coreEndpoint}/info`).then(({ data }) => {
      dispatch(
        setOwshen({
          type: "SET_OWSHEN",
          payload: {
            wallet: data.address,
            contract_address: data.owshen_contract,
            contract_abi: data.owshen_abi,
            dive_address: data.dive_contract,
            dive_abi: data.erc20_abi,
            token_contracts: data.token_contracts,
          },
        })
      );
      dispatch(setIsTest(data.isTest));
    });

    // eslint-disable-next-line react-hooks/exhaustive-deps
  };
  // [OwshenWallet.wallet, OwshenWallet.contract_address, coreEndpoint]);

  const canOpenModal = () => {
    if (isTest) {
      return setIsInprogress(true);
    }
    address ? setIsOpen(true) : toast.error("Connect your wallet first");
  };

  return (
    <>
      <TransactionModal
        transactionType="Withdraw"
        setTokenContract={setTokenContract}
        isOpen={isOpenWithdraw}
        setIsOpen={setIsOpenWithdraw}
      />
      <TransactionModal
        transactionType="send"
        tokenContract={tokenContract}
        setTokenContract={setTokenContract}
        isOpen={isOpen}
        setIsOpen={setIsOpen}
      />

      <InProgress isOpen={isInprogress} setIsOpen={setIsInprogress} />

      <div style={{ textAlign: "center" }}>
        <div className="mt-10 ">
          <Tooltip id="copy" place="top" content="Copy wallet address" />
          {/* 🌊 Owshen Wallet 🌊 */}
          {OwshenWallet.wallet && (
            <button
              data-tooltip-id="copy"
              onClick={() => copyWalletAddress(OwshenWallet.wallet)}
              className="mt-4 rounded-2xl px-3 py-2 w-52 mx-auto justify-between border border-gray-300 bg-[#BBDCFBCC] ease-in-out duration-300  flex"
            >
              {shortenAddress(OwshenWallet.wallet)}
              <img className="ml-2" src={CopyIcon} />
            </button>
          )}

          <div className="text-3xl font-bold mt-4">0.0 DIVE</div>
          <div className="text-lg mt-4">$? USD</div>
          <div className="my-8 flex justify-around w-32 mx-auto">
            <Tooltip id="send" place="bottom" content="send" />
            <button data-tooltip-id="send" onClick={canOpenModal}>
              <img src={SendIcon} />
            </button>
            <Tooltip id="swap" place="bottom" content="swap" />
            <button
              onClick={() => setIsInprogress(true)}
              data-tooltip-id="swap"
              // onClick={openWithdraw}
            >
              <img src={SwapIcon} />
            </button>
          </div>
        </div>
      </div>
      {children}
    </>
  );
};

export default Main;
