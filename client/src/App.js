import React, { useState } from "react";
import "@fontsource/jetbrains-mono"; // Defaults to weight 400
import "@fontsource/jetbrains-mono/400.css"; // Specify weight
import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import Footer from "./components/Footer";
import Header from "./components/Header";
import { Provider } from "react-redux";
import store from "./store/store";
import { WagmiProvider } from "wagmi";
import { config } from "./config";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import AllRoutes from "./components/AllRoutes";

const App = () => {
  const queryClient = new QueryClient();
  const [isDarkTheme, setIsDarkTheme] = useState(() => {
    // Parse the stored value as a boolean
    return localStorage.getItem("theme") === "true";
  });

  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <Provider store={store}>
          <div className={`h-screen ${isDarkTheme ? "dark" : ""} `}>
            <ToastContainer theme="colored" position="bottom-right" />
            <div
              className="lg:mx-72 p-8 lg:mt-14 h-5/6 min-h-full lg:min-h-[726px] flex flex-col
            bg-white dark:bg-indigo-950 dark:text-white lg:rounded-lg lg:shadow-2xl ease-in-out duration-300"
            >
              <Header
                isDarkTheme={isDarkTheme}
                setIsDarkTheme={setIsDarkTheme}
              />
              <AllRoutes />
              <Footer />
            </div>
          </div>
        </Provider>
      </QueryClientProvider>
    </WagmiProvider>
  );
};

export default App;
