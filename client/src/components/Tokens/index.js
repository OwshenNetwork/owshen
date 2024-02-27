import { Link } from "react-router-dom";
import { currencies } from "../../utils/Currencies";
import Modal from "../Modal";
import ImportTokens from "../ImportToken";
import { useState } from "react";
import { selectIsTest } from "../../store/containerSlice";
import { useSelector } from "react-redux";
import InProgress from "../Modal/InProgress";
import Main from "../Main";
import { selectUserAddress } from "../../store/containerSlice";

const Tokens = ({ tokensInfo }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [isInprogress, setIsInprogress] = useState(false);
  const address = useSelector(selectUserAddress);

  const isTest = useSelector(selectIsTest);

  return (
    <Main>
      <InProgress isOpen={isInprogress} setIsOpen={setIsInprogress} />
      <Modal title="Import tokens" isOpen={isOpen} setIsOpen={setIsOpen}>
        <ImportTokens />
      </Modal>
      <div className="received-coins-container mx-52">
        <h1 className="text-3xl text-left mb-7 font-bold">Tokens</h1>
        <ul className={`${!address ? "pointer-events-none" : ""}`}>
          {tokensInfo.map(({ name, symbol },id) => {
            return (
              <Link
                to={`${isTest ? "/" : `token/${name}`}`}
                onClick={() => (isTest ? setIsInprogress(true) : "")}
                key={id}
              >
                <li className=" flex flex-wrap mb-5 items-center  border-b-2 border-black">
                  <p className="w-1/12 text-left font-bold text-lg">
                    <img src={currencies["DIVE"].img} className="w-12" />
                  </p>
                  <div className="w-7/12 text-lg text-left">
                    <p>{name}</p> <p>0 {symbol}</p>
                  </div>

                  <div className=" w-4/12 text-right">$? USD</div>
                </li>
              </Link>
            );
          })}
        </ul>
        <p
          className="text-lg font-bold text-blue-600 cursor-pointer inline-block"
          onClick={() => (isTest ? setIsInprogress(true) : setIsOpen(true))}
        >
          +import
        </p>
      </div>
    </Main>
  );
};

export default Tokens;
