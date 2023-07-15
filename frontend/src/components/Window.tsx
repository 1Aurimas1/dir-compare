import React from "react";
import { FileType } from "../constants/constants";

interface WindowProps {
  id: number;
  type: string;
  file: string;
  showPrevious: (e: React.MouseEvent<HTMLButtonElement>, id: number) => void;
  showNext: (e: React.MouseEvent<HTMLButtonElement>, id: number) => void;
}

const Window = (props: WindowProps) => {
  return (
    <>
      <div className="w-1/2 p-4">
        <h2 className="text-lg font-semibold">File {props.id + 1}</h2>
        <div className="border border-gray-300 rounded p-4 h-64 overflow-auto">
          {props.type === FileType.Image
            ? <img src={props.file} alt="Image" />
            : <p>{props.file}</p>}
        </div>
        <div className="flex justify-end mt-4">
          <button
            className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded mr-2"
            onClick={(e) => props.showPrevious(e, props.id)}
          >
            Previous
          </button>
          <button
            className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded"
            onClick={(e) => props.showNext(e, props.id)}
          >
            Next
          </button>
        </div>
      </div>
    </>
  );
};

export default Window;
