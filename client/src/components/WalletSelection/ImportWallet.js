import { useState, useEffect } from "react";
import { useWalletSelectionApi } from "../../api/hooks/useWalletSelectionApi";
import { useMainApi } from "../../api/hooks/useMainApi";
import { selectIsOwshenWalletExist } from "../../store/containerSlice";
import { useNavigate } from "react-router-dom";
import { useSelector } from "react-redux";

const ImportWallet = () => {
  const { callImportWallet } = useWalletSelectionApi();
  const [phrases, setPhrases] = useState(Array(12).fill("")); // Initialize with empty strings
  const IsOwshenWalletExist = useSelector(selectIsOwshenWalletExist);
  const navigate = useNavigate();
  const { getInfo } = useMainApi();

  useEffect(() => {
    getInfo();
    if (IsOwshenWalletExist) {
      navigate("/");
    }
  }, [IsOwshenWalletExist, navigate, getInfo]);

  const handlePhraseChange = (index) => (e) => {
    const inputWords = e.target.value.trim().split(/\s+/);
    if (inputWords.length === 12) {
      setPhrases(inputWords);
    } else {
      const newPhrases = [...phrases];
      newPhrases[index] = e.target.value.trim();
      setPhrases(newPhrases);
    }
  };

  const importWallet = async () => {
    try {
      await callImportWallet(phrases);
      await getInfo();
    } catch (error) {
      console.error("Error while importing wallet: ", error);
    }
  };

  const btnCS =
    "border lg:w-[270px] w-full rounded-xl p-3 text-sm  my-2  ease-in-out duration-300 flex items-center justify-around bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] dark:bg-indigo-950";
  const inputCs =
    "border border-black rounded-lg py-2 w-5/6 text-center dark:bg-transparent dark:border-white";
  const inputHolderCs = "w-full flex items-center justify-between";

  return (
    <>
      <div className="text-center flex lg:h-[700px] w-full justify-center items-center ">
        <div className="w-full md:w-[570px] md:border-2 md:p-6 rounded-lg">
          <p className="text-lg  my-5">
            Please enter your 12-word mnemonic phrase to securely import your
            wallet!
          </p>
          <div className="border-2 grid md:grid-cols-3 grid-cols-2 p-3 rounded-lg gap-5">
            {phrases.map((phrase, index) => (
              <div key={index} className={inputHolderCs}>
                {index + 1}.
                <input
                  name={String(index + 1)}
                  value={phrase}
                  onChange={handlePhraseChange(index)}
                  type="text"
                  className={inputCs}
                  placeholder={`Word ${index + 1}`}
                />
              </div>
            ))}
          </div>
          <div className="flex flex-col items-center mt-4">
            <button onClick={importWallet} className={btnCS}>
              Import
            </button>
          </div>
        </div>
      </div>
    </>
  );
};

export default ImportWallet;
