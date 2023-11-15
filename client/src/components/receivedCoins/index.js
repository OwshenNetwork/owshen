import React from "react";
import axios from "axios";
import { ethers, getAddress } from "ethers";
import { useSelector } from "react-redux";
import { toBigInt } from "ethers";
import SendIcon from "../../pics/icons/send-inside.svg";
import SwapIcon from "../../pics/icons/swap-inside.svg";
import {
  selectOwshen,
  selectReceivedCoins,
  selectUserAddress,
} from "../../store/containerSlice";
import "./style.css";

const ReceivedCoinList = () => {
  const owshen = useSelector(selectOwshen);
  const address = useSelector(selectUserAddress);
  const receivedcoins = useSelector(selectReceivedCoins);

  const withdrawal = async (index, owshen, address) => {
    if (!address) return console.log("Connect your wallet first");
    const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;
    const options = {
      gasLimit: 1000000,
    };

    axios
      .get(`${coreEndpoint}/withdraw`, {
        params: { index: index },
      })
      .then(async (result) => {
        let abi = owshen.contract_abi;
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

        try {
          const amount = result.data.amount;
          const txResponse = await contract.withdraw(
            result.data.nullifier,
            proof,
            getAddress(owshen.selected_token_contract),
            toBigInt(amount),
            address,
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

  return (
    <div className="received-coins-container mx-52">
      <h1 className="text-3xl text-left mb-7 font-bold">Coins</h1>
      <ul className="">
        {receivedcoins?.map((coin, index) => (
          <li
            className=" flex flex-wrap mb-5  border-b-2 border-[#00000033]"
            key={index}
          >
            <p className="w-3/6 text-left font-bold text-lg">
              {toBigInt(coin.amount).toString() * 10 ** -18} ETH
            </p>
            <p className="w-1/6 text-lg">
              {String(coin.token).substring(0, 10)}
            </p>
            <p className="w-1/6 text-lg pl-5"> {coin.index}</p>
            <div className=" w-1/6">
              <button onClick={() => withdrawal(coin.index, owshen, address)}>
                <img alt="" src={SendIcon} />
              </button>

              <button className="ml-2">
                <img alt="" src={SwapIcon} />
              </button>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default ReceivedCoinList;
