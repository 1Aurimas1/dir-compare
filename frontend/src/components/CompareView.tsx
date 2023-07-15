import React, { useEffect, useRef, useState } from "react";
import SearchBar from "./SearchBar";
import Window from "./Window";
import { BASE_URL, FileType } from "../constants/constants";

const CompareView = () => {
  const WS_URL = BASE_URL;
  const [windows, setWindowsContent] = useState(["", ""]);
  const [types, setTypes] = useState(["", ""]);
  const [searchError, setSearchError] = useState("");
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const socket = new WebSocket(WS_URL);
    socket.onopen = function (e) {
      console.log("[open] Connection established");
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
      console.log(`[message] Data received from server`);
      const data: Blob = event.data;
      const content_blob = data.slice(4);

      const reader = new FileReader();

      reader.onload = function () {
        function resultToString(
          data: ArrayBuffer | string | undefined,
        ): string {
          if (data instanceof ArrayBuffer) {
            const decoder = new TextDecoder("utf-8");
            return decoder.decode(data);
          }
          return typeof data === "string" ? data : "";
        }

        const cmd = resultToString(reader.result?.slice(0, 4));
        const type = cmd.slice(0, 3);
        const idx = parseInt(cmd.slice(3, 4));

        const content = resultToString(reader.result?.slice(4));

        switch (type) {
          case FileType.Image:
            if (windows[idx] !== "" && types[idx] === FileType.Image) {
              URL.revokeObjectURL(windows[idx]);
            }
            types[idx] = FileType.Image;
            windows[idx] = URL.createObjectURL(content_blob);
            break;
          case FileType.Text:
            types[idx] = FileType.Text;
            windows[idx] = content;
            break;
          default:
            console.error("Unknown file type");
            break;
        }
        setTypes([...types]);
        setWindowsContent([...windows]);
      };
      reader.readAsText(data);
    };
  }, [windows, types]);

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
        {windows.map((item, index) => (
          <Window
            key={index}
            id={index}
            type={types[index]}
            file={item}
            showPrevious={showPrevious}
            showNext={showNext}
          />
        ))}
      </div>
    </div>
  );
};

export default CompareView;
