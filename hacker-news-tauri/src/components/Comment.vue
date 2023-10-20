<script setup lang="ts">
import { reactive } from "vue";
import { Item } from "../types/article";
import { invoke } from "@tauri-apps/api/tauri";
import UserModal from "./UserModal.vue";

interface Props {
    comment: Item;
}

interface State {
    commentsOpen: boolean;
    comments: Item[];
    fetching: boolean;
    error?: string;
    userVisible: boolean;
}

const props = defineProps<Props>();
const state = reactive<State>({
    commentsOpen: false,
    comments: [],
    fetching: false,
    userVisible: false,
});

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
    invoke<Item[]>("get_items", { items: props.comment.kids })
        .then((items) => (state.comments = items))
        .catch((err) => (state.error = err))
        .finally(() => (state.fetching = false));
}

function toggleText() {
    return state.commentsOpen ? "[-]" : "[+]";
}

function toggleUserView() {
    state.userVisible = !state.userVisible;
}

</script>

<template>
    <div class="talk-bubble tri-right left-top">
        <div class="comment">
            <span v-html="comment.text" />
        </div>

        <UserModal :visible="state.userVisible" :user-handle="props.comment.by"/>

        <div class="bottom">
            <div class="author">
                by
                <span @click="toggleUserView()" class="by">
                    {{ props.comment.by }}
                </span>
                 {{ props.comment.time }}
            </div>
            <div class="commentFooterContainer">
                <span
                    @click="toggleComments"
                    class="commentFooter"
                    v-if="props.comment.kids.length > 0"
                >
                    {{ toggleText() }}
                    {{ props.comment.kids.length }}
                    {{
                        props.comment.kids.length === 1 ? "comment" : "comments"
                    }}
                </span>
            </div>
        </div>

        <div v-if="state.fetching">Loading...</div>

        <div v-if="state.error" class="error">
            Failed to load comments: {{ state.error }}
        </div>

        <div v-if="state.commentsOpen" class="pointer">👉</div>

        <div v-if="state.commentsOpen" v-for="comment of state.comments">
            <Comment :comment="comment" />
        </div>
    </div>
</template>

<style scoped>
.comment {
    padding: 10px;
    overflow: auto;
    max-width: 35rem;
}

.talk-bubble {
    margin-left: 40px;
    margin-bottom: 5px;
    margin-top: 5px;
    display: inline-block;
    position: relative;
    height: auto;
    background-color: #f9fdc1;
    border-radius: 8px;
    padding: 10px;
    box-shadow: 1px 1px 2px -1px;
    color: black;
}

.commentFooter:hover {
    color: rgb(122, 14, 14);
    /* transform: scale(2, 2); */
    /* text-shadow: 1px 1px black; */
}

.tri-right.left-top:before {
    content: " ";
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
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    left: -20px;
    right: auto;
    top: 0px;
    bottom: auto;
    border: 22px solid;
    border-color: #f9fdc1 transparent transparent transparent;
}

.pointer {
    top: 40px;
    position: relative;
}
</style>
