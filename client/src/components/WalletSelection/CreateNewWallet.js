import { useState, useEffect } from "react";
import { useWalletSelectionApi } from "../../api/hooks/useWalletSelectionApi";
import { copyWalletWords } from "../../utils/helper";
import ReactLoading from "react-loading";
import { Link } from "react-router-dom";
import { useMainApi } from "../../api/hooks/useMainApi";
import { selectIsOwshenWalletExist } from "../../store/containerSlice";
import { useNavigate } from "react-router-dom";
import { useSelector } from "react-redux";
const CreateNewWallet = () => {
  const [words, setWords] = useState(null);
  const [CopyWords, setCopyWords] = useState(null);
  const [walletCreated, setWalletCreated] = useState(false);
  const IsOwshenWalletExist = useSelector(selectIsOwshenWalletExist);
  const navigate = useNavigate();

  const { getInfo } = useMainApi();
  const { generateWallet } = useWalletSelectionApi();

  const btnCS =
    "border lg:w-[270px] w-full rounded-xl p-3  my-2  ease-in-out duration-300 flex items-center justify-around bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] dark:bg-indigo-950";
  const inputCs = "border dark:border-white border-black rounded-lg py-2   w-5/6 text-center";
  const inputHolderCs = "w-full flex items-center justify-between";

  const fetchWallet = async () => {
    try {
      const response = await generateWallet();
      const words = response.words[0].split(" ");
      setCopyWords(response.words);
      setWords(words); // Assuming the response is the wallet words
    } catch (error) {
      console.error("Error fetching wallet:", error);
    } finally {
    }
  };
  useEffect(() => {
    getInfo();
    if (IsOwshenWalletExist && !walletCreated) {
      navigate("/");
    }
  });
  useEffect(() => {
    fetchWallet();
    setWalletCreated(true);
  }, []);

  useEffect(() => {
    // Function to handle the 'beforeunload' event
    const handleBeforeUnload = (event) => {
      // Cancel the event as stated by the standard.
      event.preventDefault();
      // Chrome requires returnValue to be set.
      event.returnValue = "";
    };

    // Add the event listener when the component mounts
    window.addEventListener("beforeunload", handleBeforeUnload);

    // Clean up the event listener when the component unmounts
    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  }, []);

  return (
    <>
      <div className="text-center flex h-full w-full justify-center items-center ">
        <div className="w-full md:w-[570px] md:border-2 md:p-6 rounded-lg">
          <p className="text-lg  my-5">
            Securely save these 12 phrases to access your Owshen wallet at any
            time.
          </p>
          <div className="border-2  grid md:grid-cols-3 grid-cols-2	p-3 rounded-lg gap-5">
            {words ? (
              words.map((word, i) => (
                <div className={inputHolderCs}>
                  {i + 1}.
                  <div name="1" type="text" className={inputCs}>
                    {word}
                  </div>
                </div>
              ))
            ) : (
              <div className="col-start-2 col-end-3 flex justify-center ">
                <ReactLoading
                  type="bars"
                  color="#2481D7"
                  height={200}
                  width={200}
                />
              </div>
            )}
          </div>
          <div className="flex flex-col items-center mt-4">
            <button
              onClick={() => copyWalletWords(CopyWords)}
              className="text-blue-500 my-4"
            >
              copy to clipboard
            </button>
            <Link to={"/"} className={btnCS}>
              Home
            </Link>
          </div>
        </div>
      </div>
    </>
  );
};

export default CreateNewWallet;
