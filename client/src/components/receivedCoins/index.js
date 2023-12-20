import React, { useEffect, useState } from "react";
import axios from "axios";
import { ethers, getAddress } from "ethers";
import { useSelector } from "react-redux";
import { toBigInt } from "ethers";
import { utils } from "web3";
import { toast } from "react-toastify";
import { Link } from "react-router-dom";
import Modal from "../Modal/Modal";
import SendIcon from "../../pics/icons/send-inside.png";
import SwapIcon from "../../pics/icons/swap-inside.png";
import {
  selectOwshen,
  selectReceivedCoins,
  selectUserAddress,
  selectReceivedCoinsLoading,
} from "../../store/containerSlice";
import ReactLoading from "react-loading";
import "./style.css";
import BackIcon from "../../pics/icons/left_arrow.png";
import MergIcon from "../../pics/icons/merge-icon.png";

const ReceivedCoinList = () => {
  const owshen = useSelector(selectOwshen);
  const address = useSelector(selectUserAddress);
  const receivedcoins = useSelector(selectReceivedCoins);
  const isLoading = useSelector(selectReceivedCoinsLoading);
  const [isOpen, setIsOpen] = useState(false);

  const withdrawal = async (index, owshen, address) => {
    if (!address) return console.log("Connect your wallet first");
    const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;
    const options = {
      gasLimit: 5000000,
    };

    axios
      .get(`${coreEndpoint}/withdraw`, {
        params: {
          index: index,
          address: owshen.wallet,
          desire_amount: "1",
        },
      })
      .then(async (result) => {
        let abi = owshen.contract_abi;
        let commitment = result.data.commitment;
        let provider = new ethers.BrowserProvider(window.ethereum);

        let contract = new ethers.Contract(
          owshen.contract_address,
          abi,
          provider
        );

        let signer = await provider.getSigner();
        contract = contract.connect(signer);

        const proof = [
          result.data.proof.a,
          result.data.proof.b,
          result.data.proof.c,
        ];

        const ax = utils.toBigInt(result.data.ephemeral.x);
        const ay = utils.toBigInt(result.data.ephemeral.y);

        const ephemeral = [ax, ay];
        try {
          const txResponse = await contract.withdraw(
            result.data.nullifier,
            ephemeral,
            proof,
            result.data.token,
            toBigInt(1),
            result.data.obfuscated_remaining_amount,
            address,
            commitment,
            options
          );
          console.log("Transaction response", txResponse);
          const txReceipt = await txResponse.wait();
          console.log("Transaction receipt", txReceipt);
        } catch (error) {
          console.log(error, "Error while getting withdraw flow");
        }
      });
  };
  const trueAmount = (val) => {
    return Number(toBigInt(val).toString()) / Math.pow(10, 18);
  };

  return (
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
      <Link to={"/"}>
        <div className="text-left flex mb-5 items-center">
          <img src={BackIcon} className="mx-5" />
          <h1 className="text-3xl text-left  font-bold"> Coins</h1>
        </div>
      </Link>
      {isLoading ? (
        <div className="flex justify-center">
          <ReactLoading type="bars" color="#2481D7" height={200} width={200} />
        </div>
      ) : receivedcoins?.length ? (
        <ul>
          {receivedcoins?.map((coin, index) => (
            <li
              className=" flex flex-wrap pb-1 items-center border-b-2 border-[#00000033]"
              key={index}
            >
              <p className="w-5/6 text-left font-bold text-lg">
                {trueAmount(coin.amount)} DIVE
              </p>

              <div className=" w-1/6 justify-between flex">
                <button onClick={() => setIsOpen(true)}>
                  <img
                    alt=""
                    width="34px"
                    className=" border border-gray-300 p-1.5 rounded-3xl"
                    src={MergIcon}
                  />
                </button>
                <button
                  className="ml-2"
                  onClick={() => withdrawal(coin.index, owshen, address)}
                >
                  <img alt="" src={SendIcon} />
                </button>

                <button className="ml-2">
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
  );
};

export default ReceivedCoinList;
