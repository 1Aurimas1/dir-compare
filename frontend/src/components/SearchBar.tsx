interface SearchBarProps {
  id: number;
  labelText: string;
}

const SearchBar = (props: SearchBarProps) => {
  return (
    <>
      <label htmlFor={`dirInput${props.id}`} className="text-lg font-semibold m-1">
        {props.labelText}
      </label>
      <input
        type="text"
        name={`dir${props.id}`}
        id={`dirInput${props.id}`}
        placeholder="Search"
        className="border border-gray-300 rounded-l py-2 px-4"
      />
    </>
  );
};

export default SearchBar;
