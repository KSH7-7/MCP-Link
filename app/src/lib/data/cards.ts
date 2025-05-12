export type Card = {
  id: number
  title: string
  content: string
}

export function getCards(): Promise<Card[]> {
  return new Promise((resolve) => {
    // API 호출 지연 시뮬레이션 (500ms)
    setTimeout(() => {
      resolve(cardData)
    }, 50)
  })
}

/**
 * ID로 특정 카드를 가져오는 함수
 * @param {number} id - 찾을 카드의 ID
 * @returns {Promise<Card|null>} 찾은 카드 또는 null을 반환하는 Promise
 */
export function getCardById(id: number): Promise<Card | null> {
  return new Promise((resolve) => {
    setTimeout(() => {
      const card = cardData.find((card) => card.id === id) || null
      resolve(card)
    }, 300)
  })
}
