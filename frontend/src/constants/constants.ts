const FileType = {
  Image: "img",
  Text: "txt",
} as const;
type FileType = typeof FileType[keyof typeof FileType];

const BASE_URL = "ws://localhost:8000";

export { BASE_URL, FileType };
