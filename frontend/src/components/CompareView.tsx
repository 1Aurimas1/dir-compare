import React, { useDeferredValue, useEffect, useRef, useState } from "react";
import SearchBar from "./SearchBar";
import Window from "./Window";
import { BASE_URL } from "../constants/constants";
import { CompareWindow, FileType } from "../shared/interfaces/CompareWindow";

const Command = {
  Search: "search",
  Prev: "prev",
  Next: "next",
} as const;
type Command = typeof Command[keyof typeof Command];

interface Message {
  file_type: string;
  window_idx: number;
  file_idx: number;
  file_total: number;
  content: ArrayBuffer;
}

const initCompareWindow: CompareWindow = {
  fileIdx: -1,
  fileTotal: 0,
  fileType: FileType.Text,
  content: "",
};

const KeyControls = {
  PrevWindow1: "ArrowDown",
  NextWindow1: "ArrowUp",
  PrevWindow2: "ArrowLeft",
  NextWindow2: "ArrowRight",
};

const inputBlockTime = 100;

const CompareView = () => {
  const WS_URL = BASE_URL;
  const wsRef = useRef<WebSocket | null>(null);
  const [searchError, setSearchError] = useState("");

  const [windows, setWindows] = useState<CompareWindow[]>(
    Array(2).fill(initCompareWindow),
  );

  const isInputBlocked = useRef(false);

  useEffect(() => {
    const socket = new WebSocket(WS_URL);
    if (wsRef.current === null) {
      wsRef.current = socket;
    } else {
      return;
    }

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

    const handleKeyPress = (e: KeyboardEvent) => {
      if (isInputBlocked.current) return;

      switch (e.key) {
        case KeyControls.PrevWindow1:
          showPrevious(e, 0);
          break;
        case KeyControls.NextWindow1:
          showNext(e, 0);
          break;
        case KeyControls.PrevWindow2:
          showPrevious(e, 1);
          break;
        case KeyControls.NextWindow2:
          showNext(e, 1);
          break;
        default:
          break;
      }

      isInputBlocked.current = true;
      setTimeout(() => {
        isInputBlocked.current = false;
      }, inputBlockTime);
    };

    window.addEventListener("keyup", handleKeyPress);

    return () => {
      socket.close();
      window.removeEventListener("keyup", handleKeyPress);
    };
  }, []);

  useEffect(() => {
    if (!wsRef.current) return;

    function resultToString(
      data: ArrayBuffer | string | null | undefined,
    ): string {
      //if (Array.isArray(data)) {
      //  data = new Uint8Array(data).buffer;
      //  const decoder = new TextDecoder("utf-8");
      //  return decoder.decode(data);
      //}

      return typeof data === "string" ? data : "";
    }

    function processMessageData(e: ProgressEvent<FileReader>) {
      const receivedMessage: Message = JSON.parse(
        resultToString(e.target?.result),
      );
      const idx = receivedMessage.window_idx;

      let newContent: string;
      switch (receivedMessage.file_type) {
        case FileType.Image:
          if (
            windows[idx].content !== "" &&
            windows[idx].fileType === FileType.Image
          ) {
            URL.revokeObjectURL(windows[idx].content);
          }
          newContent = URL.createObjectURL(
            new Blob([new Uint8Array(receivedMessage.content)]),
          );
          break;
        case FileType.Text:
          const data = new Uint8Array(receivedMessage.content).buffer;
          const decoder = new TextDecoder("utf-8");
          newContent = decoder.decode(data);
          break;
        default:
          console.error("Unknown file type...");
          return;
      }

      const updatedWindow = {
        fileIdx: receivedMessage.file_idx,
        fileTotal: receivedMessage.file_total,
        fileType: receivedMessage.file_type,
        content: newContent,
      };

      setWindows((prevWindows) => {
        prevWindows[idx] = updatedWindow;
        return [...prevWindows];
      });
    }

    wsRef.current.onmessage = function (event) {
      console.log(`[message] Data received from server`);
      const messageData: Blob = event.data;

      const reader = new FileReader();

      reader.onload = processMessageData;

      // IDEA: read as array buf?
      reader.readAsText(messageData);
    };

  }, [windows]);

  const showPrevious = (
    e: React.MouseEvent<HTMLButtonElement> | KeyboardEvent,
    id: number,
  ) => {
    e.preventDefault();
    if (!wsRef.current) return;

    wsRef.current.send(`op:${Command.Prev};${id}`);
  };

  const showNext = (
    e: React.MouseEvent<HTMLButtonElement> | KeyboardEvent,
    id: number,
  ) => {
    e.preventDefault();
    if (!wsRef.current) return;

    wsRef.current.send(`op:${Command.Next};${id}`);
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

    wsRef.current.send(`op:${Command.Search};dir0:${dir0};dir1:${dir1};`);
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
        {windows.map((win, i) => (
          <Window
            key={i}
            id={i}
            data={win}
            showPrevious={showPrevious}
            showNext={showNext}
          />
        ))}
      </div>
    </div>
  );
};

export default CompareView;
