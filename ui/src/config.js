import api from "./api";
import { ref } from "vue";
const CONFIG = ref({});

export default CONFIG;
export function loadConfig() {
    api.get('api/config')
        .then(res => {
            CONFIG.value = res.data;
        });
}