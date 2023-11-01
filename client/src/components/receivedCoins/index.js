import React from "react";
import axios from "axios";
import { ethers, getAddress } from "ethers";

import {
  selectOwshen,
  selectReceivedCoins,
  selectUserAddress,
} from "../../store/containerSlice";
import { useSelector } from "react-redux";
import { toBigInt } from "ethers";

import "./style.css";
import { useApprove } from "../../hooks/useApprove";

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
        //
        const amount = result.data.amount;
        const txResponse = await contract.withdraw(
          result.data.nullifier,
          proof,
          getAddress(result.data.token),
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

const ReceivedCoinList = () => {
  const owshen = useSelector(selectOwshen);
  const address = useSelector(selectUserAddress);
  const receivedcoins = useSelector(selectReceivedCoins);
  const { approve, allowance } = useApprove(
    owshen.dive_address,
    address,
    owshen.contract_address,
    owshen.dive_abi
  );

  return (
    <div className="received-coins-container">
      <ul className="list-container">
        {receivedcoins?.map((coin, index) => (
          <li className="coin-container" key={index}>
            <p>Amount: {toBigInt(coin.amount).toString() * 10 ** -18}</p>
            <p>Token Address: {String(coin.token).substring(0, 10)}</p>
            <p>Index: {coin.index}</p>
            <button
              onClick={() =>
                withdrawal(coin.index, owshen, address, approve, allowance)
              }
              className="recieved-coin-btn"
            >
              withdrawal
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default ReceivedCoinList;
