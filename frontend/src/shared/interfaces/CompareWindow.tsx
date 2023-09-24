export const FileType = {
  Image: "img",
  Text: "txt",
} as const;
export type FileType = typeof FileType[keyof typeof FileType];

export interface CompareWindow {
  fileIdx: number;
  fileTotal: number;
  fileType: FileType;
  content: string;
}

export default CompareWindow;
