<script setup lang="ts">
import { reactive } from "vue";
import { Item, ViewItems } from "../types/article";
import { invoke } from "@tauri-apps/api/tauri";

interface Props {
    comment: Item;
}

interface State {
    commentsOpen: boolean,
    comments: ViewItems,
    fetching: boolean,
    error?: string,
}

const props = defineProps<Props>();
const state = reactive<State>({ commentsOpen: false, comments: { items: [] }, fetching: false });

function toggleComments() {
    if (!state.commentsOpen) {
        getComments();
    } else {
        state.error = undefined;
    }
    state.commentsOpen = !state.commentsOpen;
}

function getComments() {
    state.fetching = true;
    state.error = undefined;
    invoke<ViewItems>("get_items", { items: props.comment.kids })
        .then(items => state.comments = items)
        .catch(err => state.error = err)
        .finally(() => state.fetching = false);
}

function toggleText() {
    return state.commentsOpen ? "[-]" : "[+]";
}
</script>

<template>
    <div class="talk-bubble tri-right left-top">
        <span v-html="comment.text" />

        <div class="bottom">
            <div class="author">by {{ props.comment.by }} {{ props.comment.time }}</div>
            <div class="commentFooterContainer">
                <span @click="toggleComments"
                    class="commentFooter"
                    v-if="props.comment.kids.length > 0">
                    {{ toggleText() }}
                    {{ props.comment.kids.length }}
                    {{ props.comment.kids.length === 1 ? "comment" : "comments" }}
                </span>
            </div>

        </div>

        <div v-if="state.fetching">
            Loading...
        </div>

        <div v-if="state.error" class="error">
            Failed to load comments: {{ state.error }}
        </div>

        <div v-if="state.commentsOpen" class="pointer">
            👉
        </div>

        <div v-if="state.commentsOpen" v-for="comment of state.comments.items">
            <Comment :comment="comment" />
        </div>
    </div>
</template>

<style scoped>
.comment {
    padding: 10px;
    background-color: antiquewhite;
    border: 1px solid black;
    border-radius: 8px;
    margin: 10px;
}

.talk-bubble {
    margin-left: 40px;
    margin-bottom: 5px;
    margin-top: 5px;
    display: inline-block;
    position: relative;
    height: auto;
    background-color: antiquewhite;
    border-radius: 8px;
    padding: 10px;
    box-shadow: 1px 1px 2px -1px;
    color: black;
}

.commentFooter:hover {
    color: fuchsia;
    /* transform: scale(2, 2); */
    /* text-shadow: 1px 1px black; */
}

.tri-right.left-top:before {
    content: ' ';
    position: absolute;
    width: 0;
    height: 0;
    left: -40px;
    right: auto;
    top: -1px;
    bottom: auto;
    border: 0px solid;
    border-color: #666 transparent transparent transparent;
}

.tri-right.left-top:after {
    content: ' ';
    position: absolute;
    width: 0;
    height: 0;
    left: -20px;
    right: auto;
    top: 0px;
    bottom: auto;
    border: 22px solid;
    border-color: antiquewhite transparent transparent transparent;
}

.pointer {
    top: 40px;
    position: relative;
}
</style>