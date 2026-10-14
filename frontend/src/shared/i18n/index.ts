import i18n from 'i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import { initReactI18next } from 'react-i18next'
import en from './locales/en/common.json'
import fr from './locales/fr/common.json'
import ru from './locales/ru/common.json'

export const languages = ['en', 'fr', 'ru'] as const
export type Language = (typeof languages)[number]

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { common: en },
      fr: { common: fr },
      ru: { common: ru },
    },
    defaultNS: 'common',
    fallbackLng: 'en',
    supportedLngs: languages,
    interpolation: { escapeValue: false },
  })

export default i18n
