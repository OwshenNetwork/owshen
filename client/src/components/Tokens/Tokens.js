import { Link } from "react-router-dom";
import { currencies } from "../../utils/Currencies";
import Modal from "../Modal/Modal";
import ImportTokens from "../ImportToken/ImportToken";
import { useState } from "react";

const Tokens = ({ tokensInfo }) => {
  const [isOpen , setIsOpen] = useState(false)
  return (
    <>
      <Modal title="Import tokens" isOpen={isOpen} setIsOpen={setIsOpen}><ImportTokens/></Modal>
      <div className="received-coins-container mx-52">
        <h1 className="text-3xl text-left mb-7 font-bold">Tokens</h1>
        <ul>
          {tokensInfo.map(({ name,symbol }) => {
            return (
              <Link to={`/${name}`}>
                <li className=" flex flex-wrap mb-5 items-center  border-b-2 border-black">
                  <p className="w-1/12 text-left font-bold text-lg">
                    <img src={currencies["ETH"].img} className="w-12" />
                  </p>
                  <div className="w-7/12 text-lg text-left">
                    <p>{name}</p> <p>0 {symbol}</p>
                  </div>

                  <div className=" w-4/12 text-right">$0.00 USD</div>
                </li>
              </Link>
            );
          })}
        </ul>
        <p className="text-lg font-bold text-blue-600 cursor-pointer" onClick={()=>setIsOpen(true)}>+import</p>
      </div>
    </>
  );
};

export default Tokens;
