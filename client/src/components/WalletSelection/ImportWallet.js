import { useState, useEffect } from "react";
import { useWalletSelectionApi } from "../../api/hooks/useWalletSelectionApi";
import { useMainApi } from "../../api/hooks/useMainApi";
import { selectIsOwshenWalletExist } from "../../store/containerSlice";
import { useNavigate } from "react-router-dom";
import { useSelector } from "react-redux";
const ImportWallet = () => {
  const { callImportWallet } = useWalletSelectionApi();
  const [phrases, setPhrases] = useState([]);
  const IsOwshenWalletExist = useSelector(selectIsOwshenWalletExist);
  const navigate = useNavigate();
  const { getInfo } = useMainApi();

  useEffect(() => {
    getInfo();
    if (IsOwshenWalletExist) {
      navigate("/");
    }
  });

  const handelPhrases = (e) => {
    setPhrases((prev) => {
      const index = parseInt(e.target.name, 10) - 1;
      const newPhrases = [...prev];
      newPhrases[index] = e.target.value;
      return newPhrases;
    });
  };
  const importWallet = async () => {
    try {
      await callImportWallet(phrases);
      await getInfo();
    } catch (error) {
      console.log(error);
    }
  };

  const btnCS =
    "border lg:w-[270px] w-full rounded-xl p-3  my-2  ease-in-out duration-300 flex items-center justify-around bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] dark:bg-indigo-950";
  const inputCs = "border border-black rounded-lg py-2 w-5/6 text-center dark:bg-transparent dark:border-white";
  const inputHolderCs = "w-full flex items-center justify-between";
  return (
    <>
      <div className="text-center flex h-full w-full justify-center items-center ">
      <div className="w-full md:w-[570px] md:border-2 md:p-6 rounded-lg">
          <p className="text-lg  my-5">
            Please enter your 12-word secret phrase to securely import your
            wallet.
          </p>
          <div className="border-2 grid md:grid-cols-3 grid-cols-2	p-3 rounded-lg gap-5">
            <div className={inputHolderCs}>
              1.
              <input
                name="1"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              2.
              <input
                name="2"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              3.
              <input
                name="3"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              4.
              <input
                name="4"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              5.
              <input
                name="5"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              6.
              <input
                name="6"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              7.
              <input
                name="7"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              8.
              <input
                name="8"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              9.
              <input
                name="9"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              10.
              <input
                name="10"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              11.
              <input
                name="11"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
            <div className={inputHolderCs}>
              12.
              <input
                name="12"
                onChange={handelPhrases}
                type="text"
                className={inputCs}
              />
            </div>
          </div>
          <div className="flex flex-col items-center mt-4">
            <button onClick={importWallet} className={btnCS}>
              import Owshen Wallet
            </button>
          </div>
        </div>
      </div>
    </>
  );
};

export default ImportWallet;
