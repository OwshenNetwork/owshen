import React, { useState } from "react";
import { useSelector } from "react-redux";
import Modal from "../Modal";
import Main from "../Main";
import {
  selectReceivedCoins,
  selectIsTest,
  selectReceivedCoinsLoading,
  selectUserAddress,
} from "../../store/containerSlice";
import ReactLoading from "react-loading";
import TransactionModal from "../Modal/TransactionModal";
import InProgress from "../Modal/InProgress";
import SendIcon from "../../pics/icons/send-inside.png";
import SwapIcon from "../../pics/icons/swap-inside.png";
import {
  getLogoByContractAddress,
  getNameByContractAddress,
} from "../../utils/Currencies";
import { getRound, trueAmount } from "../../utils/helper";
import { Tooltip } from "react-tooltip";
import { toast } from "react-toastify";
const ReceivedCoinList = () => {
  const receivedCoins = useSelector(selectReceivedCoins);
  const isLoading = useSelector(selectReceivedCoinsLoading);
  const [isOpen, setIsOpen] = useState(false);
  const [selectedCoin, SetSelectedCoin] = useState("");
  const [isOpenWithdraw, setIsOpenWithdraw] = useState(false);
  const [isDataSet, setIsDataSet] = useState(false);
  const [isInprogress, setIsInprogress] = useState(false);
  const isTest = useSelector(selectIsTest);
  const address = useSelector(selectUserAddress);

  const withdrawHandler = (coin) => {
    if (!address) {
      return toast.error(
        "Your wallet is not connected. Please connect your wallet to proceed."
      );
    }

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
      <div className=" text-center lg:max-w-[830px] w-full mx-auto">
        <Modal title="merge coins" isOpen={isOpen} setIsOpen={setIsOpen}>
          <div>
            <h3 className="text-xl font-bold mt-5 mb-3">
              Are you want merge all of your coins?
            </h3>
            <p className="bg-yellow-100 text-amber-950 text-lg px-6 m-auto inline-block py-2  border border-amber-950 ">
              Gas fee = 1 USDT
            </p>
          </div>
          <button className="border border-green-400 bg-green-200 text-green-600 rounded-lg px-6 mt-3 font-bold py-1">
            Yes
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
        ) : receivedCoins?.length ? (
          <div className="md:max-h-[310px] mb-1 overflow-y-auto max-h-[29vh] ">
            <ul>
              {receivedCoins?.map((coin, index) => (
                <li
                  className=" flex flex-wrap p-2 rounded-md mb-1 items-center border-2 bg-blue-100 dark:bg-blue-950 hover:bg-transparent dark:hover:bg-transparent ease-in-out duration-300 border-[#00000033] dark:hover:border-[#ffffff52]"
                  key={index}
                >
                  <div className="md:w-5/6 w-1/2 text-left font-bold text-lg flex items-center">
                    <img
                      className="w-8 mr-2"
                      src={getLogoByContractAddress(coin.uint_token)}
                      alt="coin"
                    />
                    <p>
                      {getRound(
                        Number(trueAmount(coin.amount, coin.uint_token))
                      )}
                      {" "}{getNameByContractAddress(coin.uint_token)}
                    </p>
                  </div>
                  <div className=" md:w-1/6 w-1/2 justify-end flex">
                    <Tooltip id="SendIcon" place="bottom" content="Withdraw" />

                    <button
                      className="ml-2"
                      onClick={() => withdrawHandler(coin)}
                      data-tooltip-id="SendIcon"
                    >
                      <img
                        alt="SendIcon"
                        className="w-10 dark:invert"
                        src={SendIcon}
                      />
                    </button>
                    <Tooltip id="SwapIcon" place="bottom" content="Swap" />
                    <button
                      onClick={() => setIsInprogress(true)}
                      className="ml-2"
                      data-tooltip-id="SwapIcon"
                    >
                      <img
                        alt="SwapIcon"
                        className="w-10 dark:invert"
                        src={SwapIcon}
                      />
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        ) : (
          <p className="text-4xl font-bold mt-28">No coins yet </p>
        )}
      </div>
    </Main>
  );
};

export default ReceivedCoinList;
