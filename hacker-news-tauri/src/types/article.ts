export interface TopStories {
  items: Item[];
  loaded?: string;
  rustArticles?: number;
  totalStories: number;
}

export interface Item {
  id: number;
  kids: number[];
  text?: string;
  url?: string;
  title?: string;
  score: number;
  time?: string;
  by: string;
  hasRust: boolean;
  viewed: boolean;
  new: boolean;
  positionChange: PositionChange;
  ty: string;
}

export interface PositionChange {
  type: "Up" | "Down" | "UnChanged";
}
