export interface ViewItems {
  items: Item[];
  loaded?: string;
  rustArticles?: number;
}

export interface Item {
  id: number;
  kids: number[];
  text?: string,
  url?: string,
  title?: string,
  score: number,
  time?: string,
  by: string,
  hasRust: boolean;
  viewed: boolean;
}

