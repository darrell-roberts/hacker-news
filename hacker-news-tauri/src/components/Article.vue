<script setup lang="ts">
import { reactive } from "vue";
import { Item, ViewItems } from "../types/article";
import { invoke } from "@tauri-apps/api/tauri";
import Comment from "./Comment.vue";
import { shell } from "@tauri-apps/api";

interface Props {
    item: Item;
    index: number;
}

interface State {
    commentsOpen: boolean;
    fetching: boolean;
    error?: string;
    comments: ViewItems;
}

const props = defineProps<Props>();
const state = reactive<State>({
    commentsOpen: false,
    fetching: false,
    comments: { items: [] },
});

const emit = defineEmits(["viewed", "url"]);

function openLink() {
    emit("viewed");
    if (props.item.url) {
        shell.open(props.item.url)
    }
}

function toggleComments() {
    if (!state.commentsOpen) {
        state.fetching = true;
        state.error = undefined;
        getComments()
            .catch(err => state.error = err)
            .finally(() => state.fetching = false);
    } else {
        state.error = undefined;
    }
    state.commentsOpen = !state.commentsOpen;
}

async function getComments() {
    emit("viewed");
    state.comments = await invoke<ViewItems>("get_items", { items: props.item.kids });
}

function toggleText() {
    return state.commentsOpen ? "[-]" : "[+]";
}

function hasRust() {
    return props.item.hasRust
}
</script>

<template>
    <div :class="{ article: true, rustArticle: hasRust(), viewed: props.item.viewed }">
        <div class="title-container">
            <div class="title">
                <span>{{ props.index + 1 }}. </span>
                <span @click="openLink"
                    v-if="props.item.url"
                    v-on:mouseover="() => emit('url', props.item.url)"
                    v-on:mouseout="() => emit('url', '')">
                    {{ props.item.title }}
                </span>
                <span v-else @click="toggleComments"
                    v-on:mouseover="() => emit('url', 'Text article')"
                    v-on:mouseout="() => emit('url', '')">
                        {{ props.item.title }}
                </span>
            </div>

            <div v-if="hasRust()">
                <img src="/rust-logo-blk.svg" class="rustBadge" />
            </div>
        </div>

        <div class="bottom">
            <div class="author">
                {{ props.item.score }} points, by {{ props.item.by }} {{ props.item.time }}
            </div>

            <div class="commentFooterContainer">
                <span @click="toggleComments"
                    class="commentFooter">
                    <span v-if="props.item.kids.length > 0">
                        {{ toggleText() }}
                        {{ props.item.kids.length }}
                        {{ props.item.kids.length === 1 ? "comment" : "comments" }}
                    </span>
                    <span v-else-if="props.item.text">{{ toggleText() }}</span>
                </span>

            </div>
        </div>

        <div v-if="state.fetching">
            Loading...
        </div>

        <div v-if="state.error" class="error">
            Failed to load comments: {{ state.error }}
        </div>

        <div v-if="state.commentsOpen && props.item.text"
            class="text-talk-bubble text-tri-right right-top">
            <span v-html="props.item.text" />
        </div>

        <div v-if="state.commentsOpen"
            v-for="comment of state.comments.items">
            <Comment :comment="comment" />
        </div>
    </div>
</template>

<style scoped>
.article {
    text-align: start;
    padding: 10px;
    margin: 5px;
    border-radius: 8px;
    background-color: #b3b1a0;
    color: black;
}

.rustArticle {
    border: 10px solid #f4c949;
}

.title {
    font-size: larger;
    font-weight: 500;
    cursor: pointer;
}

.commentFooter:hover {
    color: yellow;
    transform: scale(2, 2);
}

.title-container {
    flex-direction: row;
    display: flex;
    justify-content: space-between;
}

.rustBadge {
    width: 32px;
    height: 32px;
}

.text-talk-bubble {
    margin-right: 40px;
    margin-bottom: 5px;
    margin-top: 5px;
    display: inline-block;
    position: relative;
    height: auto;
    background-color: aliceblue;
    border-radius: 8px;
    padding: 10px;
    box-shadow: -1px 1px 2px -1px;
    color: black;
}

.text-tri-right.right-top:before {
    content: ' ';
    position: absolute;
    width: 0;
    height: 0;
    right: -40px;
    left: auto;
    top: -1px;
    bottom: auto;
    border: 0px solid;
    border-color: #666 transparent transparent transparent;
}

.text-tri-right.right-top:after {
    content: ' ';
    position: absolute;
    width: 0;
    height: 0;
    right: -20px;
    left: auto;
    top: 0px;
    bottom: auto;
    border: 22px solid;
    border-color: aliceblue transparent transparent transparent;
}

.viewed {
    background-color: #979688;
}
</style>