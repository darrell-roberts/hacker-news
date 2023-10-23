<script setup lang="ts">
import { reactive } from "vue";
import { Item } from "../types/article";
import { invoke } from "@tauri-apps/api/tauri";
import Comment from "./Comment.vue";
import UserModal from "./UserModal.vue";
import { shell } from "@tauri-apps/api";

interface Props {
    item: Item;
    index: number;
}

interface State {
    commentsOpen: boolean;
    fetching: boolean;
    error?: string;
    comments: Item[];
    userVisible: boolean;
}

const props = defineProps<Props>();
const state = reactive<State>({
    commentsOpen: false,
    fetching: false,
    comments: [],
    userVisible: false,
});

const emit = defineEmits(["viewed", "url"]);

function openLink() {
    emit("viewed");
    if (props.item.url) {
        shell.open(props.item.url);
    }
}

function toggleComments() {
    if (!state.commentsOpen) {
        state.fetching = true;
        state.error = undefined;
        getComments()
            .catch((err) => (state.error = err))
            .finally(() => (state.fetching = false));
    } else {
        state.error = undefined;
    }
    state.commentsOpen = !state.commentsOpen;
}

async function getComments() {
    emit("viewed");
    state.comments = await invoke<Item[]>("get_items", {
        items: props.item.kids,
    });
}

function toggleText() {
    return state.commentsOpen ? "[-]" : "[+]";
}

function hasRust() {
    return props.item.hasRust;
}

function positionChanged() {
    if (props.item.new) {
        return "🆕";
    }
    if (props.item.positionChange.type === "Up") {
        return "🔺";
    } else if (props.item.positionChange.type === "Down") {
        return "🔻";
    } else {
        return "";
    }
}

function toggleUserView() {
    state.userVisible = !state.userVisible;
}

function typeBadge() {
    switch (props.item.ty) {
        case "job": return "(job)";
        case "poll": return "(poll)";
        case "pollopt": return "(pollopt)";
        default: return ""
    }
}
</script>

<template>
    <div :class="{
        article: true,
        viewed: props.item.viewed,
        nonStory: props.item.ty !== 'story',
    }">

        <div class="title-container">
            <div class="title">
                <span>{{ props.index + 1 }}. </span>
                <span @click="openLink" v-if="props.item.url" v-on:mouseover="() => emit('url', props.item.url)"
                    v-on:mouseout="() => emit('url', '')">
                    {{ props.item.title }}
                </span>
                <span v-else @click="toggleComments" v-on:mouseover="() => emit('url', 'Text article')"
                    v-on:mouseout="() => emit('url', '')">
                    {{ props.item.title }}
                </span>
            </div>

            <div v-if="hasRust()" class="rustArticle">
                <img src="/rust-logo-blk.svg" class="rustBadge" />
            </div>

            <div style="display: flex;">
                <div style="margin-left: 10px">{{ typeBadge() }}</div>
                <div v-if="positionChanged() !== ''" class="positionChange">{{ positionChanged() }}</div>
            </div>

        </div>

        <UserModal :visible="state.userVisible" :user-handle="props.item.by" @close="toggleUserView()" />

        <div class="bottom">
            <div class="author">
                {{ props.item.score }} points, by
                <span class="by" @click="toggleUserView()">{{ props.item.by }}</span>
                {{ props.item.time }}

            </div>

            <div >
                <span @click="toggleComments" class="commentFooter">
                    <span v-if="props.item.kids.length > 0">
                        {{ toggleText() }}
                        {{ props.item.kids.length }}
                        {{
                            props.item.kids.length === 1
                            ? "comment"
                            : "comments"
                        }}
                    </span>
                    <span v-else-if="props.item.text">{{ toggleText() }}</span>
                </span>
            </div>

        </div>

        <div v-if="state.fetching">Loading...</div>

        <div v-if="state.error" class="error">
            Failed to load comments: {{ state.error }}
        </div>

        <div v-if="state.commentsOpen && props.item.text" class="text-talk-bubble text-tri-right right-top">
            <span v-html="props.item.text" />
        </div>

        <div v-if="state.commentsOpen" v-for=" comment  of  state.comments ">
            <Comment :comment="comment" />
        </div>
    </div>
</template>

<style scoped>
.article {
    text-align: start;
    padding: 10px;
    margin: 2px;
    border-radius: 8px;
    background-color: white;
    color: black;
    display: inline-block;
    width: 95%;
    box-shadow: 2px 1px 1px;
}

.nonStory {
    background-color: rgb(187, 187, 125);
}

.nonStory.viewed {
    background-color: rgb(149, 149, 100);
}

.rustArticle {
    display: flex;
    flex-grow: 1;
    justify-content: end;
}

.title {
    font-size: larger;
    font-weight: 500;
    cursor: pointer;
}

.commentFooter:hover {
    color: rgb(122, 14, 14);
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

.positionChange {
    width: 10px;
    height: 10px;
    margin-right: 10px;
    margin-left: 10px;
}

.text-talk-bubble {
    margin-right: 40px;
    margin-bottom: 5px;
    margin-top: 5px;
    display: inline-block;
    position: relative;
    height: auto;
    background-color: #e4f7fb;
    border-radius: 8px;
    padding: 10px;
    box-shadow: -1px 1px 2px -1px;
    color: black;
}

.text-tri-right.right-top:before {
    content: " ";
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
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    right: -20px;
    left: auto;
    top: 0px;
    bottom: auto;
    border: 22px solid;
    border-color: #e4f7fb transparent transparent transparent;
}

.viewed {
    background-color: #e1e1e1;
}
</style>
