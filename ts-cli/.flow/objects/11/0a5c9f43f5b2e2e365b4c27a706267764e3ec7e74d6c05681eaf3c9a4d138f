declare module 'fast-glob' {
  interface Options {
    cwd?: string;
    dot?: boolean;
    onlyFiles?: boolean;
    followSymbolicLinks?: boolean;
  }
  function fg(patterns: string | string[], options?: Options): Promise<string[]>;
  export default fg;
}
