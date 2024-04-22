import axios from "axios";
import { coreEndpoint } from "../../utils/helper";
import {
  setOwshen,
  setReceivedCoins,
  setReceivedCoinsLoading,
  setIsTest,
  setIsOwshenWalletExist
} from "../../store/containerSlice";
import { useDispatch } from "react-redux";
import { toast } from "react-toastify";

export const useMainApi = () => {
  const dispatch = useDispatch();

  const setChainId = async (chain_id) => {
    if (!chain_id) {
      toast.error(
        "Your wallet is not connected. Please connect your wallet to proceed."
      );
      return;
    }
    await axios
      .post(`${coreEndpoint}/set-network`, null, {
        params: { chain_id },
      })
      .then(() => {
        getInfo();
      })
      .catch((error) => {
        console.error("Error:", error);
      });
  };

  const getCoins = () => {
    dispatch(setReceivedCoinsLoading(true));
    const coinsIntervalId = setInterval(() => {
      axios.get(`${coreEndpoint}/coins`).then((result) => {
        dispatch(
          setReceivedCoins({
            type: "SET_RECEIVED_COINS",
            payload: result.data.coins,
          })
        );
        dispatch(setReceivedCoinsLoading(result.data.syncing));
      });
    }, 5000);
    return () => clearInterval(coinsIntervalId);
  };

  const getInfo = () => {
    axios
      .get(`${coreEndpoint}/info`)
      .then(({ data }) => {
        dispatch(
          setOwshen({
            type: "SET_OWSHEN",
            payload: {
              wallet: data.address,
              contract_address: data.owshen_contract,
              contract_abi: data.owshen_abi,
              dive_address: data.dive_contract,
              dive_abi: data.erc20_abi,
              token_contracts: data.token_contracts,
            },
          })
        );
        dispatch(setIsTest(data.isTest));
        dispatch(setIsOwshenWalletExist(true));
        getCoins();
      })
      .catch((error) => {
        console.error("Error fetching info:", error);
        // Optionally, display an error message to the user
        dispatch(setIsOwshenWalletExist(false));
      });
  };

  return { setChainId, getCoins, getInfo };
};
