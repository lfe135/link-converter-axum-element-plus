import { ElButton, ElDescriptions, ElDescriptionsItem, ElForm, ElFormItem, ElInput, ElLink, ElMessage, ElRow, ElTable, ElTableColumn } from "element-plus";
import { defineComponent, onMounted, ref } from "vue";

export default defineComponent({
    setup() {
        const link = ref()
        const convertedLink = ref()
        const data = ref()
        const convert = async () => {
            let res = await fetch(`${import.meta.env.VITE_BASE_URL}/convert`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ url: link.value }) })
            if (res.ok) {
                let data = await res.json()
                convertedLink.value = `${import.meta.env.VITE_BASE_URL}/${data.dst}`
                await refresh()
            } else if (res.status === 400) {
                let text = await res.text()
                ElMessage.error(text)
            }
        }
        const refresh = async () => {
            let res = await fetch(`${import.meta.env.VITE_BASE_URL}/list`)
            if (res.ok) {
                data.value = await res.json()
            } else {
                let text = await res.text()
                ElMessage.error(text)
            }
        }
        onMounted(refresh)
        return () =>
            <div style={{ width: '80vw' }}>
                <ElRow>
                    <ElForm inline>
                        <ElFormItem label={"原链接"}>
                            <ElInput type={"textarea"} autosize resize={"horizontal"} v-model={link.value} inputStyle={{ maxWidth: '50vw' }}></ElInput>
                        </ElFormItem>
                        <ElFormItem>
                            <ElButton onClick={convert}>转换</ElButton>
                        </ElFormItem>
                    </ElForm>
                </ElRow>
                <ElRow>
                    <ElDescriptions>
                        <ElDescriptionsItem label={"转换的链接"}><ElLink>{convertedLink.value}</ElLink></ElDescriptionsItem>
                    </ElDescriptions>
                </ElRow>
                <ElTable data={data.value}>
                    <ElTableColumn label={"原链接"} prop={"src"}></ElTableColumn>
                    <ElTableColumn label={"转换的链接"} formatter={(row) => `${import.meta.env.VITE_BASE_URL}/${row.dst}`}></ElTableColumn>
                    <ElTableColumn label={"点击次数"} prop={"count"}></ElTableColumn>
                </ElTable>
            </div>
    }
})