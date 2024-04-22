import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import ReceivedCoinList from "../components/ReceivedCoins/index";
import WalletSelection from "../components/WalletSelection"
import NotFound from "../components/NotFound";
import ImportWallet from "../components/WalletSelection/ImportWallet";
import CreateNewWallet from "../components/WalletSelection/CreateNewWallet";

const AllRoutes = () => {
  const tokenInfo = [{ name: "Owshen", symbol: "DIVE" }];

  return (
    <Router basename="/">
      <Routes>
        {tokenInfo.map((name) => (
          <Route key={name} path={`/`} element={<ReceivedCoinList />} />
        ))}
        <Route path="/walletSelection" element={<WalletSelection/>} />
        <Route path="/walletSelection/importWallet" element={<ImportWallet/>} />
        <Route path="/walletSelection/createNewWallet" element={<CreateNewWallet/>} />
        <Route path="*" element={<NotFound />} /> {/* Catch-all route for 404 */}
      </Routes>
    </Router>
  );
};

export default AllRoutes;
