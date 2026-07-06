import { defineStore } from 'pinia'
import { http } from '../api/http'

export interface Conversation {
  id: string
  title: string
}

export interface Message {
  role: 'user' | 'assistant'
  content: string
  steps?: AgentStep[]
}

export interface AgentStep {
  type: 'status' | 'tool_start' | 'tool_result'
  title: string
  detail?: string
}

interface ApiMessage {
  id: string
  role: 'user' | 'assistant' | string
  content: string
}

export interface AiProvider {
  id: string
  name: string
  provider_type: string
  default_model?: string
  active: boolean
}

export interface AiModel {
  id: string
  provider_id: string
  name: string
  active: boolean
}

export const useChatStore = defineStore('chat', {
  state: () => ({
    conversations: [] as Conversation[],
    providers: [] as AiProvider[],
    models: [] as AiModel[],
    activeConversationId: '',
    messages: [] as Message[],
    messagesByConversation: {} as Record<string, Message[]>,
    inFlightConversationIds: [] as string[],
    abortControllers: {} as Record<string, AbortController | undefined>,
    provider: localStorage.getItem('server-assistant-chat-provider') || 'ollama',
    model: localStorage.getItem('server-assistant-chat-model') || 'llama3',
    sending: false
  }),
  getters: {
    modelsForProvider: (state) => state.models.filter((model) => model.provider_id === state.provider && model.active),
    activeConversationSending: (state) => state.inFlightConversationIds.includes(state.activeConversationId),
    isConversationSending: (state) => (conversationId: string) => state.inFlightConversationIds.includes(conversationId)
  },
  actions: {
    async loadConversations() {
      const { data } = await http.get('/api/conversations')
      this.conversations = data
      if (!this.activeConversationId && this.conversations.length) {
        this.activeConversationId = this.conversations[0].id
        await this.loadMessages(this.activeConversationId)
      }
    },
    async loadMessages(conversationId: string) {
      if (this.messagesByConversation[conversationId]?.length) {
        this.messages = this.messagesByConversation[conversationId]
        this.sending = this.activeConversationSending
        return
      }

      const { data } = await http.get(`/api/chat/${conversationId}/messages`)
      const messages = data
        .filter((message: ApiMessage) => message.role === 'user' || message.role === 'assistant')
        .map((message: ApiMessage) => ({
          role: message.role as 'user' | 'assistant',
          content: message.content
        }))
      this.messagesByConversation[conversationId] = messages
      this.messages = messages
      this.sending = this.activeConversationSending
    },
    async loadAiOptions() {
      const [providersResponse, modelsResponse] = await Promise.all([
        http.get('/api/admin/providers'),
        http.get('/api/admin/models')
      ])
      this.providers = providersResponse.data.filter((provider: AiProvider) => provider.active)
      this.models = modelsResponse.data.filter((model: AiModel) => model.active)

      if (this.providers.length && !this.providers.some((provider) => provider.id === this.provider)) {
        const preferred =
          this.providers.find((provider) => provider.name.toLowerCase() === 'ollama') ||
          this.providers.find((provider) => provider.provider_type.toLowerCase() === 'ollama') ||
          this.providers[0]
        this.provider = preferred.id
        this.model = preferred.default_model || this.models.find((model) => model.provider_id === preferred.id)?.name || ''
        persistChatSelection(this.provider, this.model)
      }
    },
    selectProvider(providerId: string) {
      this.provider = providerId
      const provider = this.providers.find((item) => item.id === providerId)
      this.model = provider?.default_model || this.models.find((model) => model.provider_id === providerId)?.name || ''
      persistChatSelection(this.provider, this.model)
    },
    selectModel(model: string) {
      this.model = model
      persistChatSelection(this.provider, this.model)
    },
    async newConversation() {
      const { data } = await http.post('/api/conversations', { title: 'Novo chat' })
      this.conversations.unshift(data)
      this.activeConversationId = data.id
      this.messagesByConversation[data.id] = []
      this.messages = []
      this.sending = false
    },
    async selectConversation(conversationId: string) {
      this.activeConversationId = conversationId
      this.sending = this.activeConversationSending
      await this.loadMessages(conversationId)
    },
    async renameConversation(conversationId: string, title: string) {
      const cleanTitle = title.trim()
      if (!cleanTitle) return
      const { data } = await http.patch(`/api/conversations/${conversationId}`, { title: cleanTitle })
      const conversation = this.conversations.find((item) => item.id === conversationId)
      if (conversation) conversation.title = data.title
    },
    async deleteConversation(conversationId: string) {
      await http.delete(`/api/conversations/${conversationId}`)
      this.conversations = this.conversations.filter((conversation) => conversation.id !== conversationId)
      delete this.messagesByConversation[conversationId]
      this.abortControllers[conversationId]?.abort()
      delete this.abortControllers[conversationId]
      this.inFlightConversationIds = this.inFlightConversationIds.filter((id) => id !== conversationId)
      if (this.activeConversationId === conversationId) {
        this.activeConversationId = this.conversations[0]?.id ?? ''
        this.messages = this.activeConversationId ? (this.messagesByConversation[this.activeConversationId] ?? []) : []
        this.sending = this.activeConversationSending
      }
    },
    async send(content: string) {
      if (!this.activeConversationId) await this.newConversation()
      const conversationId = this.activeConversationId
      if (this.inFlightConversationIds.includes(conversationId)) return

      const controller = new AbortController()
      this.abortControllers[conversationId] = controller
      this.inFlightConversationIds = [...this.inFlightConversationIds, conversationId]
      this.sending = this.activeConversationSending

      const targetMessages = this.messagesByConversation[conversationId] ?? this.messages
      this.messagesByConversation[conversationId] = targetMessages
      if (this.activeConversationId === conversationId) this.messages = targetMessages

      targetMessages.push({ role: 'user', content })
      const assistantMessage: Message = {
        role: 'assistant',
        content: '',
        steps: [{ type: 'status', title: 'Pensando...' }]
      }
      targetMessages.push(assistantMessage)
      try {
        const response = await fetch(`/api/chat/${conversationId}/messages/stream`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            provider: this.provider,
            model: this.model,
            content
          }),
          signal: controller.signal
        })

        if (!response.ok || !response.body) {
          const error = await response.json().catch(() => null)
          throw new Error(error?.error ?? 'Erro ao chamar o provider.')
        }

        const reader = response.body.getReader()
        const decoder = new TextDecoder()
        let buffer = ''

        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          buffer += decoder.decode(value, { stream: true })
          const events = buffer.split('\n\n')
          buffer = events.pop() ?? ''

          for (const event of events) {
            const parsed = parseSseEvent(event)
            if (parsed.event === 'delta') {
              assistantMessage.content += parsed.data
            }
            if (parsed.event === 'status') {
              assistantMessage.steps = [
                ...(assistantMessage.steps ?? []),
                { type: 'status', title: parsed.data }
              ]
            }
            if (parsed.event === 'tool_start') {
              const data = parseJsonEvent(parsed.data)
              assistantMessage.steps = [
                ...(assistantMessage.steps ?? []),
                {
                  type: 'tool_start',
                  title: `Chamando tool: ${data?.name ?? 'ferramenta'}`,
                  detail: formatJson(data?.arguments)
                }
              ]
            }
            if (parsed.event === 'tool_result') {
              const data = parseJsonEvent(parsed.data)
              assistantMessage.steps = [
                ...(assistantMessage.steps ?? []),
                {
                  type: 'tool_result',
                  title: `Tool finalizada: ${data?.name ?? 'ferramenta'}`,
                  detail: formatJson(data?.result)
                }
              ]
            }
            if (parsed.event === 'error') {
              throw new Error(parsed.data)
            }
          }
        }
      } catch (error: any) {
        if (error?.name === 'AbortError') {
          assistantMessage.steps = [
            ...(assistantMessage.steps ?? []),
            { type: 'status', title: 'Resposta interrompida pelo usuario.' }
          ]
          if (!assistantMessage.content) {
            assistantMessage.content = 'Resposta interrompida.'
          }
          return
        }

        assistantMessage.content = formatChatError(error?.message ?? error?.response?.data?.error)
      } finally {
        delete this.abortControllers[conversationId]
        this.inFlightConversationIds = this.inFlightConversationIds.filter((id) => id !== conversationId)
        this.sending = this.activeConversationSending
      }
    },
    cancelResponse(conversationId?: string) {
      const targetConversationId = conversationId ?? this.activeConversationId
      this.abortControllers[targetConversationId]?.abort()
    }
  }
})

function parseSseEvent(raw: string) {
  const lines = raw.split('\n')
  const event = lines.find((line) => line.startsWith('event:'))?.replace('event:', '').trim() ?? 'message'
  const data = lines
    .filter((line) => line.startsWith('data:'))
    .map((line) => {
      const value = line.slice(5)
      return value.startsWith(' ') ? value.slice(1) : value
    })
    .join('\n')

  return { event, data }
}

function parseJsonEvent(data: string) {
  try {
    return JSON.parse(data)
  } catch {
    return null
  }
}

function formatJson(value: unknown) {
  if (value === undefined || value === null) return ''
  if (typeof value === 'string') return value
  return JSON.stringify(value, null, 2)
}

function persistChatSelection(provider: string, model: string) {
  localStorage.setItem('server-assistant-chat-provider', provider)
  localStorage.setItem('server-assistant-chat-model', model)
}

function formatChatError(message?: string) {
  const raw = message || 'Erro ao chamar o provider.'
  const lower = raw.toLowerCase()

  if (raw.includes('HTTP 401')) {
    return [
      'A OpenAI recusou a API key configurada.',
      '',
      'Confira se a chave completa foi colada no provider OpenAI. Copiar a chave mascarada do dashboard, tipo sk-...abcd, nao funciona.'
    ].join('\n')
  }

  if (raw.includes('HTTP 429')) {
    return [
      'A OpenAI recusou por limite ou billing.',
      '',
      'Confira saldo, limite mensal do projeto e rate limit da conta.'
    ].join('\n')
  }

  if (raw.includes('HTTP 404') || lower.includes('model_not_found')) {
    return [
      'A OpenAI nao encontrou o modelo selecionado.',
      '',
      'Verifique se o nome do modelo esta correto e disponivel para sua conta.'
    ].join('\n')
  }

  if (lower.includes('api.openai.com') && (lower.includes('falha de conexao') || lower.includes('timeout') || lower.includes('error sending request'))) {
    return [
      'Nao consegui conectar na OpenAI.',
      '',
      'A API key pode estar correta, mas o backend nao conseguiu completar a conexao com api.openai.com.',
      '',
      'Verifique firewall, proxy, antivirus/SSL inspection ou reinicie o backend para carregar a configuracao atual.',
      '',
      'Para usar local, selecione o provider Ollama no topo do chat.'
    ].join('\n')
  }

  if (raw.includes('Connection refused') || raw.includes('tcp connect error') || lower.includes('falha de conexao') || raw.includes('error sending request')) {
    return [
      'Nao consegui conectar no provider selecionado.',
      '',
      'Verifique se a URL/base_url esta correta e se o servico esta rodando.'
    ].join('\n')
  }

  return raw
}
