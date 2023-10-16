<script setup lang="ts">
import Article from "./components/Article.vue";
import { ViewItems, Item } from "./types/article";
import { onMounted, reactive } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import Tooltip from "./components/Tooltip.vue";

interface State {
  viewItems: ViewItems,
  fetching: boolean,
  error: string | undefined,
  filtered: boolean,
  url?: string
}

const state = reactive<State>({
    viewItems: { items: [] },
    fetching: false,
    error: undefined,
    filtered: false
});

onMounted(() => {
  getItems();
});

function getItems() {
  state.fetching = true;
  invoke<ViewItems>("get_stories")
    .then(mergeViewed)
    .catch((err) => state.error = err)
    .finally(() => state.fetching = false);
}

function toggleFilter() {
  state.filtered = !state.filtered;
}

function applyFilter(item: Item) {
  if (state.filtered) {
    return item.hasRust;
  } else {
    return true;
  }
}

function mergeViewed(items: ViewItems) {
    const viewed = state.viewItems.items.filter(item => item.viewed).map(item => item.id);

    for (const item of items.items) {
        if (viewed.includes(item.id)) {
            item.viewed = true;
        }
    }
    state.viewItems = items;
}
</script>

<template>
  <div class="container">
    <div v-if="state.error">Failed to load stories: {{ state.error }}</div>

    <div class="articles">
      <div v-for="(item, index) in state.viewItems.items" :key="item.id">
        <Article
            :item="item"
            :index="index"
            v-if="applyFilter(item)"
            @viewed="() => item.viewed = true"
            @url="(url) => state.url = url"
        />
      </div>
    </div>

    <span class="url">{{ state.url }}</span>

    <div class="status-line">
        <Tooltip content="Reload">
        <div>
            <div v-if="state.fetching">Loading...</div>
            <div v-else @click="getItems()" class="status-action">
                {{ state.viewItems.loaded }}
            </div>
        </div>
        </Tooltip>
        <Tooltip content="Filter Rust">
            <div @click="toggleFilter()" class="status-action">
                Rust articles: {{ state.viewItems.rustArticles }}
            </div>
        </Tooltip>
    </div>
  </div>
</template>

<style scoped>
.articles {
  overflow: auto;
  height: calc(100vh - 65px);
}

.footer {
  margin-top: 10px;
  height: 50px;
}

.status-line {
  color: gray;
  font-size: small;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  margin-left: 5px;
  margin-right: 5px;
}

.status-action {
  cursor: pointer;

}

.status-action:hover {
  color: white;
}

.url {
    text-align: left;
    color: white;
    text-overflow: ellipsis;
    overflow: hidden;
    text-align: left;
    text-wrap: nowrap;
    height: 25px;
    width: 95vw;
    margin-left: 5px;
    white-space: nowrap;
}
</style>
