import { coreEndpoint, SwitchNetwork } from "../../utils/helper";
import axios from "axios";
import { toast } from "react-toastify";
import { useMainApi } from "../../api/hooks/useMainApi";

export const useSelectNetworkApi = () => {
  const { setChainId } = useMainApi();

  const setChainIdOnCore = async (newChainId, val, chainId) => {
    if (newChainId !== chainId) {
      try {
        await SwitchNetwork(val);
      } catch (error) {
        console.error("Failed to switch network:", error);
        toast.error(`Failed to switch network to ${val}`);
      }
    }
    // if (newChainId !== chainId && val) {
    //   toast.error(`Please change your wallet network to ${val}`);
    // }
    let chain_id = newChainId;
    try {
      const response = await axios.post(`${coreEndpoint}/set-network`, null, {
        params: { chain_id },
      });
      console.log("Response:", response.data);
      setChainId(chain_id);
      return response.data;
    } catch (error) {
      console.error("Error:", error);
      toast.error("Failed to set chain ID on core");
    }
  };
  return { setChainIdOnCore };
};
