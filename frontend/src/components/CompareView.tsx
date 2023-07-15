import React, { useEffect, useRef, useState } from "react";
import SearchBar from "./SearchBar";
import Window from "./Window";

const CompareView = () => {
  const WS_URL = "ws://localhost:8000";
  const [files, setFiles] = useState(["", ""]);
  const [types, setTypes] = useState(["", ""]);
  const [file1, setFile1] = useState("");
  const [file2, setFile2] = useState("");
  //const [file1_bin, setFile1_bin] = useState<ArrayBuffer | null>(null);
  //const [file2_bin, setFile2_bin] = useState<ArrayBuffer | null>(null);
  //const wsRef = useRef<Socket | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const socket = new WebSocket(WS_URL);
    socket.onopen = function (e) {
      console.log("[open] Connection established");
      console.log("Sending to server");
    };

    socket.onclose = function (event) {
      if (event.wasClean) {
        console.log(
          `[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`,
        );
      } else {
        // e.g. server process killed or network down
        // event.code is usually 1006 in this case
        console.log("[close] Connection died");
      }
    };

    socket.onerror = function (error) {
      console.log(`[error]`);
    };

    if (wsRef.current === null) {
      wsRef.current = socket;
    }

    return () => {
      socket.close();
    };
  }, []);

  useEffect(() => {
    if (!wsRef.current) return;
    wsRef.current.onmessage = function (event) {
      console.log(`[message] Data received from server: ${event.data}`);
      const data: Blob = event.data;
      const file_type = data.slice(0, 4);
      const content = data.slice(4);
      const reader = new FileReader();
      reader.onload = function () {
        const type = reader.result;
        for (let i = 0; i < files.length; i++) {
          if (type === "img" + i) {
            types[i] = "img";
            setTypes([...types]);
            files[i] = URL.createObjectURL(content);
            setFiles([...files]);
          } else if (type === "txt" + i) {
            types[i] = "img";
            setTypes([...types]);
            files[i] = URL.createObjectURL(content);
            console.log(files);
            setFiles([...files]);
            console.log(files);
          }
        }
        //const json = JSON.parse(text);
        console.log("Received JSON:", type);
      };
      reader.readAsText(file_type);
      // cleanup resource URL.revokeObjectURL
      //const blob_URL = URL.createObjectURL(content);
      //setFile1(blob_URL);
    };
  }, [files, types]);
  //}, [file1, file2]);

  const [searchError, setSearchError] = useState("");

  const showPrevious = (e: React.MouseEvent<HTMLButtonElement>, id: number) => {
    e.preventDefault();
    if (!wsRef.current) return;
    wsRef.current.send(`op:prev;${id}`);
  };

  const showNext = (e: React.MouseEvent<HTMLButtonElement>, id: number) => {
    e.preventDefault();
    if (!wsRef.current) return;
    wsRef.current.send(`op:next;${id}`);
  };

  const handleSearch = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!wsRef.current) return;
    const target = e.target as typeof e.target & {
      dir0: { value: string };
      dir1: { value: string };
    };
    const dir0 = target.dir0.value;
    const dir1 = target.dir1.value;
    if (dir0.trim() === "" && dir1.trim() === "") {
      setSearchError("Atleast one search box should be filled");
      return;
    }
    setSearchError("");

    wsRef.current.send(`op:search;dir0:${dir0};dir1:${dir1};`);
  };

  return (
    <div>
      <div className="flex justify-left m-4">
        <form onSubmit={handleSearch}>
          <SearchBar id={0} labelText={"First Dir"} />
          <SearchBar id={1} labelText={"Second Dir"} />
          <button
            className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded-r"
            type="submit"
          >
            Search
          </button>
          {searchError && <div className="text-red-500">{searchError}</div>}
        </form>
      </div>
      <div className="flex">
        {files.map((item, index) => 
        (
        <>
        {item}
        {index}
          <Window
            id={index}
            type={types[index]}
            file={item}
            showPrevious={showPrevious}
            showNext={showNext}
          />
          </>
        ))}
        {
          /*<Window
          id={0}
          file={file1}
          showPrevious={showPrevious}
          showNext={showNext}
        />
        <Window
          id={1}
          file={file2}
          showPrevious={showPrevious}
          showNext={showNext}
        />
        */
        }
      </div>
    </div>
  );
};

export default CompareView;
