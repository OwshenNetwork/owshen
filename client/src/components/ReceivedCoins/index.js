import React, { useState } from "react";
import { useSelector } from "react-redux";
import { toBigInt } from "ethers";
import Modal from "../Modal/Modal";
import Main from "../Main";
import {
  selectReceivedCoins,
  selectIsTest,
  selectReceivedCoinsLoading,
} from "../../store/containerSlice";
import ReactLoading from "react-loading";
import TransactionModal from "../Modal/TransactionModal";
import InProgress from "../Modal/InProgress";

import MergIcon from "../../pics/icons/merge-icon.png";
import SendIcon from "../../pics/icons/send-inside.png";
import SwapIcon from "../../pics/icons/swap-inside.png";
import {
  getLogoByContractAddress,
  getNameByContractAddress,
} from "../../utils/Currencies";
import { getRound } from "../../utils/helper";

const ReceivedCoinList = () => {
  const receivedcoins = useSelector(selectReceivedCoins);
  const isLoading = useSelector(selectReceivedCoinsLoading);
  const [isOpen, setIsOpen] = useState(false);
  const [selectedCoin, SetSelectedCoin] = useState("");
  const [isOpenWithdraw, setIsOpenWithdraw] = useState(false);
  const [isDataSet, setIsDataSet] = useState(false);
  const [isInprogress, setIsInprogress] = useState(false);
  const isTest = useSelector(selectIsTest);

  const trueAmount = (val) => {
    return Number(toBigInt(val).toString()) / Math.pow(10, 18);
  };
  const withdrawHandler = (coin) => {
    if (isTest) {
      return setIsInprogress(true);
    }
    SetSelectedCoin(coin);
    setIsOpenWithdraw(true);
    setIsDataSet(true);
  };

  return (
    <Main>
      <InProgress isOpen={isInprogress} setIsOpen={setIsInprogress} />
      <TransactionModal
        transactionType="Withdraw"
        isOpen={isOpenWithdraw}
        setIsOpen={setIsOpenWithdraw}
        selectedCoin={selectedCoin}
        isDataSet={isDataSet}
      />
      <div className="received-coins-container mx-52">
        <Modal title="merge coins" isOpen={isOpen} setIsOpen={setIsOpen}>
          <div>
            <h3 className="text-xl font-bold mt-5 mb-3">
              Are you want merge all of your coins?
            </h3>
            <p className="bg-yellow-100 text-amber-950 text-lg px-6 m-auto inline-block py-2  border border-amber-950 ">
              Gas fee = 1 usdt
            </p>
          </div>
          <button className="border border-green-400 bg-green-200 text-green-600 rounded-lg px-6 mt-3 font-bold py-1">
            {" "}
            yes
          </button>
        </Modal>
        {isLoading ? (
          <div>
            <div className="flex justify-center">
              <ReactLoading
                type="bars"
                color="#2481D7"
                height={200}
                width={200}
              />
            </div>
            <div>
              {typeof isLoading === "number"
                ? `${Number(isLoading * 100).toFixed(2)}% loading...`
                : "Loading..."}
            </div>
          </div>
        ) : receivedcoins?.length ? (
          <ul>
            {receivedcoins?.map((coin, index) => (
              <li
                className=" flex flex-wrap pb-1 items-center border-b-2 border-[#00000033]"
                key={index}
              >
                <div className="w-5/6 text-left font-bold text-lg flex items-center">
                  <img
                    className="w-8 mr-2"
                    src={getLogoByContractAddress(coin.uint_token)}
                  />
                  <p>
                    {String(trueAmount(coin.amount)).includes(".")
                      ? getRound(Number(trueAmount(coin.amount)))
                      : `${getRound(trueAmount(coin.amount))}.0`}{" "}
                    {getNameByContractAddress(coin.uint_token)}
                  </p>
                </div>
                {/* <p className="w-1/6 text-lg">
                {String(coin.uint_token).substring(0, 10)}
              </p>
              <p className="w-1/6 text-lg pl-5"> 
              {coin.index}
              </p> */}
                <div className=" w-1/6 justify-between flex">
                  <button
                    onClick={
                      () => setIsInprogress(true)
                      // setIsOpen(true)
                    }
                  >
                    <img
                      alt=""
                      width="34px"
                      className=" border border-gray-300 p-1.5 rounded-3xl"
                      src={MergIcon}
                    />
                  </button>
                  <button
                    className="ml-2"
                    onClick={() => setIsInprogress(true)}
                    //   withdrawal(coin.index, owshen, address);
                    //   SetSelectedCoin(coin.index);
                    //   setIsOpenWithdraw(true);
                    // }}
                  >
                    <img alt="" src={SendIcon} />
                  </button>

                  <button
                    // onClick={() => withdrawHandler(coin)}
                    onClick={() => setIsInprogress(true)}
                    className="ml-2"
                  >
                    <img alt="" src={SwapIcon} />
                  </button>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-4xl font-bold mt-28">No coins yet </p>
        )}
      </div>
    </Main>
  );
};

export default ReceivedCoinList;
