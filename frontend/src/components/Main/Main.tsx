import { useEffect, useRef, useState } from "react";
//import io from "socket.io-client";
import { Socket } from "engine.io-client";

const Main = () => {
  const [file1, setFile1] = useState("");
  const socket = new Socket("localhost:8000", { transports: ["websocket"] });
  socket.on("open", () => {
    console.log("opened");
    //setTimeout(() => {
    //  socket.send("a".repeat(255));
    //}, 5000);
    //socket.on("message", (data) => {
    //  console.log(data);
    //});
    //socket.on("close", () => {});
  });
  socket.on("message", (data) => {
    console.log("received");
    console.log(data);
    setFile1(data);
  });
  socket.on("close", () => {
    console.log("closed");
  });
  //const socketURL = "localhost:8000";
  ////useEffect(() => {
  //const socket = io(socketURL, {
  //  transports: ["websocket"],
  //});

  //socket.on("connect_error", (e) => {
  //  console.log("connect error: ", e);
  //});
  ////socket.connect();

  //socket.on("connect", () => {
  //  console.log("Connect event");
  //});
  //socket.on("connected", () => {
  //  console.log("Connected event");
  //});

  //socket.on("data", (message) => {
  //  console.log(message);
  //});
  //socket.on("message", (message) => {
  //  console.log(message);
  //});

  //socket.on("disconnect", () => {
  //  console.log("Disconnected event");
  //});
  //setTimeout(() => { socket.send("ayaya"); }, 5000);

  //return () => {
  //  console.log("Unmounted");
  //  socket.disconnect();
  //};
  //}, []);
  const [val, setVal] = useState("");
  function handleClick(e) {
    console.log("ayaya");
    e.preventDefault();
    const test = "ayaya".repeat(100);
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
    // Perform search based on the searchQuery
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

  {
    /*(
<div className="grid grid-cols-2 gap-4">
  <div className="flex flex-col">
    <div className="bg-gray-200 h-64">
      Window 1
    </div>
    <div className="mt-4 flex justify-center">
      <button className="px-4 py-2 bg-blue-500 text-white rounded">Previous</button>
      <button className="px-4 py-2 bg-blue-500 text-white rounded ml-2">Next</button>
    </div>
  </div>

  <div className="flex flex-col">
    <div className="bg-gray-200 h-64">
      Window 2
    </div>
    <div className="mt-4 flex justify-center">
      <button className="px-4 py-2 bg-blue-500 text-white rounded">Previous</button>
      <button className="px-4 py-2 bg-blue-500 text-white rounded ml-2">Next</button>
    </div>
  </div>
</div>

  );*/
  }
  {
    /*(
    <div className="grid grid-cols-4 gap-4 h-screen m-12">
      <div className="col-span-2 border-solid border-2 border-sky-500">
        Dir1
      </div>
      <div className="col-span-2 border-solid border-2 border-sky-500">
        Dir2
      </div>
      <button className="h-10 bg-[#f1f1f1]">&laquo; Previous</button>
      <button className="h-10 bg-[#04AA6D]">Next &raquo;</button>
      <button className="h-10 bg-[#f1f1f1]">&laquo; Previous</button>
      <button className="h-10 bg-[#04AA6D]">Next &raquo;</button>
      <button className="h-10 bg-[#04AA6D]">Next &raquo;</button>
    </div>
  );*/
  }
  {
    /*(
    <div className="grid grid-cols-2 grid-rows-12 gap-4 m-10">
      <div
        onClick={handleClick}
        className="scroll-smooth break-all row-span-5 border-solid border-2 border-sky-500"
      >
        {val}
      </div>
      <div className="row-start-1 row-end-10 border-solid border-2 border-sky-500">
        02
      </div>
      <div>a</div>
    </div>
  );*/
  }
  {
    /*<div className="flex flex-row gap-4 m-10">
      <div
        onClick={handleClick}
        className="flex-initial w-64 border-solid border-2 border-sky-500"
      >
        {val}
      </div>
      <div className="flex-initial w-64 border-solid border-2 border-sky-500">
        02
      </div>
      <div className=" border-solid border-2 border-sky-500">
        03
      </div>
      <h1>ayaya</h1>
    </div>*/
  }
};

export default Main;
