import { useEffect, useRef, useState } from "react";

const Main = () => {
  const [file1, setFile1] = useState("");
  const socket = new WebSocket("ws://localhost:8000");

  socket.addEventListener("open", () => {
    console.log("opened");
    });
  socket.addEventListener("message", (data) => {
    console.log("received");
    console.log(data);
    setFile1(data);
  });
  socket.addEventListener("close", () => {
    console.log("closed");
  });

  const [val, setVal] = useState("");
  function handleClick(e) {
    console.log("handleClick");
    e.preventDefault();
    const test = "test_1".repeat(100);
    setVal(test);
    socket.send("GET");
  }

  const nameForm = useRef(null);

  const [currentFile, setCurrentFile] = useState(1);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchError, setSearchError] = useState("");

  const showPrevious = () => {
    setCurrentFile((prevFile) => prevFile - 1);
  };

  const showNext = () => {
    setCurrentFile((prevFile) => prevFile + 1);
  };

  const handleSearch = (e) => {
    e.preventDefault();
    console.log("Performing search for:", searchQuery);
    //const form = nameForm.current;
    //console.log(form);
    const dir1 = e.target.dir1.value;
    const dir2 = e.target.dir2.value;
    if (dir1.trim() === "" && dir2.trim() === "") {
      setSearchError("Atleast one search box should be filled");
      return;
    }
    setSearchError("");

    socket.send(`op:search;dir1:${dir1};dir2:${dir2};`);
    console.log("sent");
  };

  return (
    <div>
      <div className="flex justify-left m-4">
        <form onSubmit={handleSearch}>
          <label htmlFor="dirInput1" className="text-lg font-semibold m-1">
            First dir
          </label>
          <input
            type="text"
            name="dir1"
            id="dirInput1"
            placeholder="Search"
            className="border border-gray-300 rounded-l py-2 px-4"
          />
          <label htmlFor="dirInput2" className="text-lg font-semibold m-1">
            Second dir
          </label>
          <input
            type="text"
            name="dir2"
            id="dirInput2"
            placeholder="Search"
            className="border border-gray-300 rounded-l py-2 px-4"
          />
          <button
            className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded-r"
            type="submit"
          >
            Search
          </button>
        </form>
      </div>

      {searchError && <div className="text-red-500">{searchError}</div>}

      <div className="flex">
        <div className="w-1/2 p-4">
          <h2 className="text-lg font-semibold">File 1</h2>
          <div className="border border-gray-300 rounded p-4 h-64 overflow-auto">
            {file1}
          </div>
          <div className="flex justify-end mt-4">
            <button
              className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded mr-2"
              onClick={showPrevious}
              disabled={currentFile === 1}
            >
              Previous
            </button>
            <button
              className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded"
              onClick={showNext}
              disabled={currentFile === 2}
            >
              Next
            </button>
          </div>
        </div>

        <div className="w-1/2 p-4">
          <h2 className="text-lg font-semibold">File 2</h2>
          <div className="border border-gray-300 rounded p-4 h-64 overflow-auto">
            {/* File 2 content here */}
          </div>
          <div className="flex justify-end mt-4">
            <button
              className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded mr-2"
              onClick={showPrevious}
              disabled={currentFile === 1}
            >
              Previous
            </button>
            <button
              className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded"
              onClick={showNext}
              disabled={currentFile === 2}
            >
              Next
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Main;
