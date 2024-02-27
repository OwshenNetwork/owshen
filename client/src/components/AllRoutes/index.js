import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import ReceivedCoinList from "../ReceivedCoins/index";

const AllRoutes = () => {
  const tokenInfo = [{ name: "Owshen", symbol: "DIVE" }];

  return (
    <Router basename="/">
      <Routes>
        {tokenInfo.map((name) => (
          <Route key={name} path={`/`} element={<ReceivedCoinList />} />
        ))}
      </Routes>
    </Router>
  );
};

export default AllRoutes;
