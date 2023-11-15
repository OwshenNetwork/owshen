import React from "react";
import Main from "./components/main";
import "@fontsource/jetbrains-mono"; // Defaults to weight 400
import "@fontsource/jetbrains-mono/400.css"; // Specify weight

const App = () => {
  return (
    <div className="h-screen overflow-y-hidden">
      <div className="lg:mx-72 p-8 mt-14 h-5/6 bg-white rounded-lg shadow-2xl">
        <Main />
      </div>
    </div>
  );
};

export default App;
