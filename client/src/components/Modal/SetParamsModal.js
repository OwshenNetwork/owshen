import { useState } from "react";
import Modal from ".";

const SetParamsModal = ({ isOpen, setIsOpen }) => {
  const [file, setFile] = useState("");

  const setFileHandler = (e) => {
    // Access the selected file(s)
    const files = e.target.files;
    // Assuming you're interested in the first file selected
    if (files.length > 0) {
      const selectedFile = files[0];
      // console.log(file.name); // This will log the file name
      // If you need to handle the file content, you can use FileReader API
      // For example, to read the file as text:
      const reader = new FileReader();
      const fileContent = reader.readAsText(selectedFile);
      // console.log(fileContent);
      setFile(fileContent)
    }
  };

  return (
    <Modal title="Set params" isOpen={isOpen} setIsOpen={setIsOpen}>
      <div className="h-60">
        <label className="block mt-10 font-medium text-gray-900 dark:text-white text-lg">
          add your file to set params
        </label>
        <input
          className="block w-3/4 text-sm text-gray-900 mx-auto mt-10 border border-gray-300 rounded-lg cursor-pointer bg-gray-50 dark:text-gray-400 focus:outline-none dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400"
          id="multiple_files"
          type="file"
          multiple
          // value={file}
          onChange={setFileHandler}
          accept=".zkey" // Ensure this is the correct file type or extension
        />
        <button className="mt-14 bg-white p-2 rounded text-black">
          send params
        </button>
      </div>
    </Modal>
  );
};

export default SetParamsModal;
