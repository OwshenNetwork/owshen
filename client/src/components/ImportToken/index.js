const ImportTokens = () => {
  return (
    <>
      <div>
        <div className="w-3/4 mx-auto">
          <p className="mt-5  ">Token contract address</p>
          <input
            className="border border-black mt-1 rounded py-2 w-full"
            type="text"
          />
        </div>
        <button className="border border-blue-400 bg-blue-200 text-blue-600 rounded-lg px-6 mt-3 font-bold py-1">Import</button>
      </div>
    </>
  );
};

export default ImportTokens;
