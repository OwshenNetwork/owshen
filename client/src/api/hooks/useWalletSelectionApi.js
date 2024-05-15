import { coreEndpoint } from "../../utils/helper";
import axios from "axios";
import { toast } from "react-toastify";

export const useWalletSelectionApi = () => {
  const generateWallet = async () => {
    try {
      const response = await axios.post(
        `${coreEndpoint}/init`,
        {
          type: "Generate",
        },
        {
          headers: {
            "Content-Type": "application/json",
          },
        }
      );
      toast.success("your owshen wallet created successfully");
      return response.data;
    } catch (error) {
      console.error("Error:", error);
      toast.error("Failed to set chain ID on core");
    }
  };
  const callImportWallet = async (enteredWords) => {
    if (enteredWords.length !== 12) {
      return toast.error(
        "Please enter all 12 words of your secret phrase to proceed."
      );
    }
    const words = [enteredWords.join(" ")];
    try {
      const response = await axios.post(
        `${coreEndpoint}/init`,
        {
          type: "Import",
          words,
        },
        {
          headers: {
            "Content-Type": "application/json",
          },
        }
      );
      toast.success("your owshen wallet Imported successfully");
      return response.data;
    } catch (error) {
      console.error("Error:", error);
      toast.error("Failed to import wallet data");
    }
  };

  return { generateWallet, callImportWallet };
};
