import axios from "axios";
import { coreEndpoint } from "../../utils/helper";
import {
  setOwshen,
  setReceivedCoins,
  setReceivedCoinsLoading,
  setIsTest,
} from "../../store/containerSlice";
import { useDispatch } from "react-redux";
import { toast } from "react-toastify"; // Assuming you're using react-toastify for toast notifications

export const useMainApi = () => {
  const dispatch = useDispatch();

  const setChainId = async (chain_id) => {
    if (!chain_id) {
      toast.error("Please connect your wallet");
      return;
    }
    await axios
      .post(`${coreEndpoint}/set-network`, null, {
        params: { chain_id },
      })
      .then((response) => {
        console.log("Response:", response.data);
        GetCoins();
        GetInfo();
      })
      .catch((error) => {
        console.error("Error:", error);
      });
  };

  const GetCoins = () => {
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

  const GetInfo = () => {
    axios.get(`${coreEndpoint}/info`).then(({ data }) => {
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
    });
  };

  return { setChainId, GetCoins, GetInfo };
};
