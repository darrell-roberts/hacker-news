export interface TopStories {
    items: Item[];
    loaded?: string;
    rustArticles?: number;
    totalStories: number;
}

export interface Link {
    link_ref: string;
    name: string;
}

export type RichText =
    | { type: "text"; content: string }
    | { type: "char"; content: string }
    | { type: "link"; content: Link }
    | { type: "paragraph" }
    | { type: "italic"; content: string }
    | { type: "bold"; content: string }
    | { type: "code"; content: string };

export interface Item {
    id: number;
    kids: number[];
    text: RichText[];
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
